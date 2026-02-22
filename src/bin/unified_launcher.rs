use napcat_backend::UnifiedLauncher;
use tracing::{info, error};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("🚀 启动统一服务 (NapCat Backend + Open-LLM-VTuber)...");
    
    // 检查环境变量
    info!("🔍 检查环境变量...");
    let required_vars = ["DATABASE_URL", "LLM_API_KEY", "LLM_BASE_URL", "LLM_MODEL"];
    for var in &required_vars {
        if env::var(var).is_err() {
            eprintln!("❌ 环境变量 {} 未设置", var);
            eprintln!("💡 请确保已配置环境变量，例如在 ~/.config/napcat-backend/secrets.env 中");
            std::process::exit(1);
        }
    }
    
    info!("✅ 环境变量检查通过");
    
    // 创建统一启动器
    let mut launcher = match UnifiedLauncher::new().await {
        Ok(launcher) => {
            info!("✅ 统一启动器创建成功");
            launcher
        }
        Err(e) => {
            error!("❌ 统一启动器创建失败: {}", e);
            std::process::exit(1);
        }
    };
    
    info!("✅ 系统初始化完成，所有组件已加载");
    info!("💡 按 Ctrl+C 停止服务");
    
    // 启动服务
    if let Err(e) = launcher.start_services().await {
        error!("❌ 服务启动失败: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
