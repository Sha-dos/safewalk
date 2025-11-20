mod server;

pub use server::*;

use tokio::process::Command;

pub async fn start_ap() {
    let _ = Command::new("ip")
        .args(["addr", "flush", "dev", "wlan0"])
        .output()
        .await;

    let _ = Command::new("ip")
        .args(["addr", "add", "10.0.0.1/24", "dev", "wlan0"])
        .output()
        .await;

    let _ = Command::new("ip")
        .args(["link", "set", "wlan0", "up"])
        .output()
        .await;

    let hostapd_config = r#"
interface=wlan0
ssid=safewalk_ap
hw_mode=g
channel=6
wmm_enabled=1
auth_algs=1
ignore_broadcast_ssid=0
wpa=2
wpa_passphrase=pass12345
wpa_key_mgmt=WPA-PSK
rsn_pairwise=CCMP
"#;

    let mut hostapd = Command::new("hostapd")
        .arg("-B")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start hostapd");

    if let Some(mut stdin) = hostapd.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(hostapd_config.as_bytes()).await.unwrap();
    }

    let _ = Command::new("dnsmasq")
        .args([
            "--interface=wlan0",
            "--bind-interfaces",
            "--dhcp-range=10.0.0.10,10.0.0.200,12h",
        ])
        .spawn();
}
