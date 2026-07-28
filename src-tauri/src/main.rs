// LiteNote 可执行程序入口
// 生产环境禁用 Windows 控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    litenote_lib::run()
}
