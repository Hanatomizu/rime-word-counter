// Windows 发布版不额外弹出控制台窗口；调试版保留，方便看 println!/日志。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    rime_word_counter_gui_lib::run()
}
