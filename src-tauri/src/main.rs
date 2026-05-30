// 在发布模式下防止在 Windows 上出现额外的控制台窗口，请勿删除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    vessel_lib::run()
}
