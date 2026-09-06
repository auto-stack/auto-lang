// Plan 532 W3:嵌套塔里程碑测试(cargo t3 大版本升级专用,附录 B 二次裁定)。
//
// 分层定位(镜像 466 churn 层"平时排除、特定闸门运行"先例):
//   日常档 t / 全量档 tf·ta 经 nextest default-filter 显式排除本测试
//   (名字 aavm2_t3_tower_milestone 进两档 filter);仅里程碑档
//   cargo t3(nextest-t3.toml,不过滤)运行。
//   测试体另有 T3_MILESTONE env 自守门——裸 cargo test / 漏配 filter
//   时秒退,三重防误触发小时级运行。
//
// 完整里程碑形态(release 二进制,塔运行分钟-小时级):
//   cargo build --release -p auto
//   T3_MILESTONE=1 cargo t3
//
// 判据:scripts/aavm_tower_check.sh(R0==O1==O2==O3 逐字节,分阶重试
// 容忍宿主静默空输出存量)退出码 0;runner 输出全量透传 --nocapture。
#[cfg(feature = "test-vm-files")]
#[test]
fn test_aavm2_t3_tower_milestone() {
    if std::env::var("T3_MILESTONE").map(|v| v != "0" && !v.is_empty()).unwrap_or(false) == false {
        eprintln!(
            "skipped: aavm2_t3_tower_milestone 是里程碑档测试(小时级)——\
             大版本升级时以 T3_MILESTONE=1 cargo t3 运行"
        );
        return;
    }
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let script = root.join("scripts").join("aavm_tower_check.sh");
    let output = std::process::Command::new("bash")
        .arg(&script)
        .current_dir(&root)
        .output()
        .expect("spawn bash scripts/aavm_tower_check.sh");
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "嵌套塔里程碑验收未通过(见上方 runner 输出;首个分歧位按阶×语料交叉报告)"
    );
}
