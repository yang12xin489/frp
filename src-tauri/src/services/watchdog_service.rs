use std::io::{BufRead, BufReader};

#[cfg(unix)]
fn kill_group(g_pid: i32) {
    unsafe {
        let _ = libc::kill(-g_pid, libc::SIGTERM);
    }
    thread::sleep(Duration::from_millis(300));
    unsafe {
        let _ = libc::kill(-g_pid, libc::SIGKILL);
    }
}

#[cfg(unix)]
fn kill_pid(pid: i32) {
    unsafe {
        let _ = libc::kill(pid, libc::SIGTERM);
    }
    thread::sleep(Duration::from_millis(200));
    unsafe {
        let _ = libc::kill(pid, libc::SIGKILL);
    }
}

#[cfg(windows)]
fn kill_tree_win(root_pid: u32) {
    use std::collections::{HashMap, VecDeque};
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_TERMINATE,
    };

    unsafe {
        // 枚举所有进程，构建 ppid -> [pid] 映射
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return;
        }

        let mut pe: PROCESSENTRY32W = std::mem::zeroed();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
        if Process32FirstW(snap, &mut pe) != 0 {
            loop {
                let pid = pe.th32ProcessID;
                let ppid = pe.th32ParentProcessID;
                children.entry(ppid).or_default().push(pid);
                if Process32NextW(snap, &mut pe) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);

        // 收集整棵子树（含根）
        let mut to_kill = Vec::new();
        let mut q = VecDeque::new();
        q.push_back(root_pid);
        while let Some(p) = q.pop_front() {
            to_kill.push(p);
            if let Some(v) = children.get(&p) {
                for &c in v {
                    q.push_back(c);
                }
            }
        }

        // 自底向上终止
        for &pid in to_kill.iter().rev() {
            let h = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if !h.is_null() {
                // 尝试终止并稍等
                let _ = TerminateProcess(h, 1);
                let _ = WaitForSingleObject(h, 50);
                CloseHandle(h);
            }
        }
    }
}

/// 作为独立进程运行；只通过 stdin 接受主进程发来的“目标”命令：
/// SET PG <pid>    （Unix: 记住进程组）
/// SET PID <pid>   （Windows 或 Unix 备用：按 PID 杀）
/// CLEAR           （清空目标）
/// EOF             （主进程死亡）→ 根据最后一次 SET 执行清理 → 退出
pub fn run() -> ! {
    #[derive(Clone, Copy)]
    enum Target {
        None,
        #[cfg(unix)]
        GPid(i64),
        Pid(i64),
    }
    let mut tgt = Target::None;

    let mut reader = BufReader::new(std::io::stdin());
    let mut line = String::new();

    while let Ok(n) = reader.read_line(&mut line) {
        if n == 0 {
            // EOF：父进程已死亡，执行清理
            match tgt {
                Target::None => {}
                #[cfg(unix)]
                Target::GPid(x) => kill_group(x as i32),
                Target::Pid(x) => {
                    #[cfg(unix)]
                    {
                        kill_pid(x as i32);
                    }
                    #[cfg(windows)]
                    {
                        kill_tree_win(x as u32);
                    }
                }
            }
            std::process::exit(0);
        }
        let cmd = line.trim();
        if cmd.is_empty() {
            line.clear();
            continue;
        }

        let mut it = cmd.split_whitespace();
        match (it.next(), it.next(), it.next()) {
            #[cfg(unix)]
            (Some("SET"), Some("PG"), Some(pid)) => {
                if let Ok(v) = pid.parse::<i64>() {
                    tgt = Target::GPid(v);
                }
            }
            (Some("SET"), Some("PID"), Some(pid)) => {
                if let Ok(v) = pid.parse::<i64>() {
                    tgt = Target::Pid(v);
                }
            }
            (Some("CLEAR"), _, _) => {
                tgt = Target::None;
            }
            _ => {}
        }
        line.clear();
    }

    std::process::exit(0);
}
