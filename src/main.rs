use std::mem::zeroed;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, Thread32First, Thread32Next,
    PROCESSENTRY32W, TH32CS_SNAPPROCESS, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

#[derive(Debug, Clone)]
pub struct ProcessCpuSample {
    pub pid: u32,
    pub name: String,
    pub thread_count: u32,
    pub kernel_time_us: u64,
    pub user_time_us: u64,
}

pub struct ThreadSampler;

impl ThreadSampler {
    pub fn collect_snapshots() -> Vec<ProcessCpuSample> {
        let mut results = Vec::new();

        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snap == INVALID_HANDLE_VALUE {
                return results;
            }

            let mut pe: PROCESSENTRY32W = zeroed();
            pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snap, &mut pe) != 0 {
                loop {
                    let len = pe.szExeFile.iter().position(|&c| c == 0).unwrap_or(pe.szExeFile.len());
                    let name = String::from_utf16_lossy(&pe.szExeFile[..len]);
                    let pid = pe.th32ProcessID;

                    let mut k_time = 0u64;
                    let mut u_time = 0u64;

                    let h_proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
                    if h_proc != 0 as HANDLE {
                        let mut creation = zeroed();
                        let mut exit = zeroed();
                        let mut kernel = zeroed();
                        let mut user = zeroed();

                        if GetProcessTimes(h_proc, &mut creation, &mut exit, &mut kernel, &mut user) != 0 {
                            k_time = ((kernel.dwHighDateTime as u64) << 32 | kernel.dwLowDateTime as u64) / 10;
                            u_time = ((user.dwHighDateTime as u64) << 32 | user.dwLowDateTime as u64) / 10;
                        }
                        CloseHandle(h_proc);
                    }

                    results.push(ProcessCpuSample {
                        pid,
                        name,
                        thread_count: pe.cntThreads,
                        kernel_time_us: k_time,
                        user_time_us: u_time,
                    });

                    if Process32NextW(snap, &mut pe) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snap);
        }

        results
    }

    pub fn enumerate_threads_for_pid(target_pid: u32) -> Vec<u32> {
        let mut tids = Vec::new();
        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snap == INVALID_HANDLE_VALUE {
                return tids;
            }

            let mut te: THREADENTRY32 = zeroed();
            te.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;

            if Thread32First(snap, &mut te) != 0 {
                loop {
                    if te.th32OwnerProcessID == target_pid {
                        tids.push(te.th32ThreadID);
                    }
                    if Thread32Next(snap, &mut te) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snap);
        }
        tids
    }
}

fn main() {
    println!("[ThreadPulse] Native Windows NT Execution & Thread Sampler");
    println!("Scanning active process execution times...\n");

    let mut samples = ThreadSampler::collect_snapshots();
    samples.sort_by(|a, b| (b.kernel_time_us + b.user_time_us).cmp(&(a.kernel_time_us + a.user_time_us)));

    println!("{:<8} {:<30} {:<10} {:<15} {:<15}", "PID", "NAME", "THREADS", "KERNEL(ms)", "USER(ms)");
    println!("{:-<80}", "");

    for proc in samples.iter().take(15) {
        println!(
            "{:<8} {:<30} {:<10} {:<15} {:<15}",
            proc.pid,
            if proc.name.len() > 28 { &proc.name[..28] } else { &proc.name },
            proc.thread_count,
            proc.kernel_time_us / 1000,
            proc.user_time_us / 1000
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_snapshots_returns_active_processes() {
        let list = ThreadSampler::collect_snapshots();
        assert!(!list.is_empty(), "Process list should not be empty");
        let current_pid = std::process::id();
        let current_proc = list.iter().find(|p| p.pid == current_pid);
        assert!(current_proc.is_some(), "Current process must be present in snapshot");
    }

    #[test]
    fn test_enumerate_threads_for_current_process() {
        let current_pid = std::process::id();
        let threads = ThreadSampler::enumerate_threads_for_pid(current_pid);
        assert!(!threads.is_empty(), "Current process must contain at least one thread");
    }
}
