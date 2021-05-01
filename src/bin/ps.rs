use procfs;


fn main() {
    let ps = procfs::process::Process::myself().unwrap();
    let tps = procfs::ticks_per_second().unwrap();

    println!("{: >5} {: <8} {: >8} {}", "PID", "TTY", "TIME", "CMD");

    let tty = format!("pty/{}", ps.stat.tty_nr().1);
    for prc in procfs::process::all_processes().unwrap() {
        let total_time = (prc.stat.utime + prc.stat.stime) as f32 / (tps as f32);
        println!("{: >5} {: <8} {: >8} {}", prc.stat.pid, tty, total_time, prc.stat.comm );
    }
}
