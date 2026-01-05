//! Process information from /proc filesystem.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: i32,
    pub ppid: i32,
    pub uid: u32,
    pub user: String,
    pub name: String,
    pub cmdline: String,
    pub exe: PathBuf,
    pub state: ProcessState,
    pub cpu_percent: f32,
    pub memory_rss: u64,
    pub memory_vms: u64,
    pub threads: u32,
    pub nice: i8,
    pub start_time: u64,
    pub io: Option<IoStats>,
    pub cgroup: Option<String>,
    pub namespace: Option<NamespaceInfo>,
}

#[derive(Debug, Clone)]
pub struct IoStats {
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_syscalls: u64,
    pub write_syscalls: u64,
}

#[derive(Debug, Clone)]
pub struct NamespaceInfo {
    pub pid_ns: u64,
    pub net_ns: u64,
    pub mnt_ns: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Sleeping,
    DiskSleep,
    Zombie,
    Stopped,
    TracingStop,
    Dead,
    Unknown,
}

impl ProcessState {
    pub fn from_char(c: char) -> Self {
        match c {
            'R' => ProcessState::Running,
            'S' => ProcessState::Sleeping,
            'D' => ProcessState::DiskSleep,
            'Z' => ProcessState::Zombie,
            'T' => ProcessState::Stopped,
            't' => ProcessState::TracingStop,
            'X' | 'x' => ProcessState::Dead,
            _ => ProcessState::Unknown,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ProcessState::Running => "R",
            ProcessState::Sleeping => "S",
            ProcessState::DiskSleep => "D",
            ProcessState::Zombie => "Z",
            ProcessState::Stopped => "T",
            ProcessState::TracingStop => "t",
            ProcessState::Dead => "X",
            ProcessState::Unknown => "?",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ProcessState::Running => "Running",
            ProcessState::Sleeping => "Sleeping",
            ProcessState::DiskSleep => "Disk Sleep",
            ProcessState::Zombie => "Zombie",
            ProcessState::Stopped => "Stopped",
            ProcessState::TracingStop => "Tracing",
            ProcessState::Dead => "Dead",
            ProcessState::Unknown => "Unknown",
        }
    }
}

pub struct ProcessCollector {
    prev_cpu_times: HashMap<i32, (u64, u64)>, // pid -> (utime+stime, total_time)
    prev_total_time: u64,
}

impl ProcessCollector {
    pub fn new() -> Self {
        Self {
            prev_cpu_times: HashMap::new(),
            prev_total_time: 0,
        }
    }

    pub fn collect(&mut self) -> Vec<Process> {
        let mut processes = Vec::new();

        // Get total CPU time
        let total_time = self.get_total_cpu_time();
        let delta_total = total_time.saturating_sub(self.prev_total_time);

        // Read /proc directory
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Some(name_str) = name.to_str() {
                    if let Ok(pid) = name_str.parse::<i32>() {
                        if let Some(proc) = self.read_process(pid, delta_total) {
                            processes.push(proc);
                        }
                    }
                }
            }
        }

        self.prev_total_time = total_time;

        // Fallback mock data if no processes found (non-Linux)
        if processes.is_empty() {
            return self.mock_processes();
        }

        processes
    }

    fn get_total_cpu_time(&self) -> u64 {
        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            if let Some(line) = stat.lines().next() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 && parts[0] == "cpu" {
                    let user: u64 = parts[1].parse().unwrap_or(0);
                    let nice: u64 = parts[2].parse().unwrap_or(0);
                    let system: u64 = parts[3].parse().unwrap_or(0);
                    let idle: u64 = parts[4].parse().unwrap_or(0);
                    return user + nice + system + idle;
                }
            }
        }
        0
    }

    fn read_process(&mut self, pid: i32, delta_total: u64) -> Option<Process> {
        let proc_path = format!("/proc/{}", pid);

        // Read /proc/[pid]/stat
        let stat = fs::read_to_string(format!("{}/stat", proc_path)).ok()?;
        let (name, stat_parts) = parse_stat(&stat)?;

        // Parse stat fields
        let state = stat_parts.get(0)?.chars().next().map(ProcessState::from_char)?;
        let ppid: i32 = stat_parts.get(1)?.parse().ok()?;
        let utime: u64 = stat_parts.get(11)?.parse().ok()?;
        let stime: u64 = stat_parts.get(12)?.parse().ok()?;
        let nice: i8 = stat_parts.get(16)?.parse().ok()?;
        let threads: u32 = stat_parts.get(17)?.parse().ok()?;
        let start_time: u64 = stat_parts.get(19)?.parse().ok()?;

        // Calculate CPU percentage
        let proc_time = utime + stime;
        let cpu_percent = if let Some(&(prev_time, _)) = self.prev_cpu_times.get(&pid) {
            let delta_proc = proc_time.saturating_sub(prev_time);
            if delta_total > 0 {
                (delta_proc as f32 / delta_total as f32) * 100.0 * num_cpus() as f32
            } else {
                0.0
            }
        } else {
            0.0
        };
        self.prev_cpu_times.insert(pid, (proc_time, delta_total));

        // Read /proc/[pid]/statm for memory
        let (memory_rss, memory_vms) = if let Ok(statm) = fs::read_to_string(format!("{}/statm", proc_path)) {
            let parts: Vec<&str> = statm.split_whitespace().collect();
            let vms: u64 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0) * 4096;
            let rss: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0) * 4096;
            (rss, vms)
        } else {
            (0, 0)
        };

        // Read /proc/[pid]/cmdline
        let cmdline = fs::read_to_string(format!("{}/cmdline", proc_path))
            .ok()
            .map(|s| s.replace('\0', " ").trim().to_string())
            .unwrap_or_default();

        // Read exe path
        let exe = fs::read_link(format!("{}/exe", proc_path)).unwrap_or_default();

        // Read UID from /proc/[pid]/status
        let (uid, user) = read_uid(&proc_path);

        // Read I/O stats
        let io = read_io_stats(&proc_path);

        // Read cgroup
        let cgroup = fs::read_to_string(format!("{}/cgroup", proc_path))
            .ok()
            .and_then(|s| s.lines().next().map(|l| l.to_string()));

        // Read namespaces
        let namespace = read_namespaces(&proc_path);

        Some(Process {
            pid,
            ppid,
            uid,
            user,
            name,
            cmdline,
            exe,
            state,
            cpu_percent,
            memory_rss,
            memory_vms,
            threads,
            nice,
            start_time,
            io,
            cgroup,
            namespace,
        })
    }

    fn mock_processes(&self) -> Vec<Process> {
        vec![
            Process {
                pid: 1,
                ppid: 0,
                uid: 0,
                user: "root".into(),
                name: "systemd".into(),
                cmdline: "/usr/lib/systemd/systemd".into(),
                exe: PathBuf::from("/usr/lib/systemd/systemd"),
                state: ProcessState::Sleeping,
                cpu_percent: 0.1,
                memory_rss: 12_500_000,
                memory_vms: 168_000_000,
                threads: 1,
                nice: 0,
                start_time: 0,
                io: None,
                cgroup: Some("0::/init.scope".into()),
                namespace: None,
            },
            Process {
                pid: 1234,
                ppid: 1,
                uid: 1000,
                user: "user".into(),
                name: "firefox".into(),
                cmdline: "/usr/lib/firefox/firefox".into(),
                exe: PathBuf::from("/usr/lib/firefox/firefox"),
                state: ProcessState::Running,
                cpu_percent: 15.5,
                memory_rss: 512_000_000,
                memory_vms: 4_000_000_000,
                threads: 80,
                nice: 0,
                start_time: 1000,
                io: Some(IoStats {
                    read_bytes: 50_000_000,
                    write_bytes: 10_000_000,
                    read_syscalls: 50000,
                    write_syscalls: 10000,
                }),
                cgroup: Some("0::/user.slice/user-1000.slice".into()),
                namespace: None,
            },
            Process {
                pid: 5678,
                ppid: 1,
                uid: 1000,
                user: "user".into(),
                name: "code".into(),
                cmdline: "/usr/share/code/code".into(),
                exe: PathBuf::from("/usr/share/code/code"),
                state: ProcessState::Sleeping,
                cpu_percent: 3.2,
                memory_rss: 256_000_000,
                memory_vms: 1_500_000_000,
                threads: 25,
                nice: 0,
                start_time: 2000,
                io: None,
                cgroup: None,
                namespace: None,
            },
            Process {
                pid: 100,
                ppid: 1,
                uid: 0,
                user: "root".into(),
                name: "sshd".into(),
                cmdline: "/usr/sbin/sshd -D".into(),
                exe: PathBuf::from("/usr/sbin/sshd"),
                state: ProcessState::Sleeping,
                cpu_percent: 0.0,
                memory_rss: 5_000_000,
                memory_vms: 15_000_000,
                threads: 1,
                nice: 0,
                start_time: 500,
                io: None,
                cgroup: Some("0::/system.slice/sshd.service".into()),
                namespace: None,
            },
            Process {
                pid: 9999,
                ppid: 1234,
                uid: 1000,
                user: "user".into(),
                name: "Web Content".into(),
                cmdline: "/usr/lib/firefox/firefox -contentproc".into(),
                exe: PathBuf::from("/usr/lib/firefox/firefox"),
                state: ProcessState::Running,
                cpu_percent: 8.3,
                memory_rss: 128_000_000,
                memory_vms: 500_000_000,
                threads: 10,
                nice: 0,
                start_time: 1500,
                io: None,
                cgroup: None,
                namespace: None,
            },
        ]
    }
}

fn parse_stat(stat: &str) -> Option<(String, Vec<&str>)> {
    // Format: pid (comm) state ppid ...
    // comm can contain spaces and parentheses
    let start = stat.find('(')?;
    let end = stat.rfind(')')?;
    let name = stat[start + 1..end].to_string();
    let rest = &stat[end + 2..];
    let parts: Vec<&str> = rest.split_whitespace().collect();
    Some((name, parts))
}

fn read_uid(proc_path: &str) -> (u32, String) {
    if let Ok(status) = fs::read_to_string(format!("{}/status", proc_path)) {
        for line in status.lines() {
            if line.starts_with("Uid:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(uid_str) = parts.get(1) {
                    if let Ok(uid) = uid_str.parse::<u32>() {
                        // Try to get username
                        let user = get_username(uid).unwrap_or_else(|| uid.to_string());
                        return (uid, user);
                    }
                }
            }
        }
    }
    (0, "unknown".into())
}

fn get_username(uid: u32) -> Option<String> {
    if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(file_uid) = parts[2].parse::<u32>() {
                    if file_uid == uid {
                        return Some(parts[0].to_string());
                    }
                }
            }
        }
    }
    None
}

fn read_io_stats(proc_path: &str) -> Option<IoStats> {
    let io = fs::read_to_string(format!("{}/io", proc_path)).ok()?;
    let mut stats = IoStats {
        read_bytes: 0,
        write_bytes: 0,
        read_syscalls: 0,
        write_syscalls: 0,
    };

    for line in io.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 {
            let value: u64 = parts[1].trim().parse().unwrap_or(0);
            match parts[0] {
                "read_bytes" => stats.read_bytes = value,
                "write_bytes" => stats.write_bytes = value,
                "syscr" => stats.read_syscalls = value,
                "syscw" => stats.write_syscalls = value,
                _ => {}
            }
        }
    }

    Some(stats)
}

fn read_namespaces(proc_path: &str) -> Option<NamespaceInfo> {
    let ns_path = format!("{}/ns", proc_path);
    let pid_ns = read_ns_inode(&format!("{}/pid", ns_path))?;
    let net_ns = read_ns_inode(&format!("{}/net", ns_path))?;
    let mnt_ns = read_ns_inode(&format!("{}/mnt", ns_path))?;

    Some(NamespaceInfo {
        pid_ns,
        net_ns,
        mnt_ns,
    })
}

fn read_ns_inode(path: &str) -> Option<u64> {
    let link = fs::read_link(path).ok()?;
    let s = link.to_string_lossy();
    // Format: type:[inode]
    let start = s.find('[')?;
    let end = s.find(']')?;
    s[start + 1..end].parse().ok()
}

fn num_cpus() -> usize {
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        cpuinfo.lines().filter(|l| l.starts_with("processor")).count()
    } else {
        1
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
