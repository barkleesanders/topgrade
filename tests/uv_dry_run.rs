#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, process::Command};

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "Black-box CLI test must spawn Topgrade and inspect nonzero exits; its internal executor is not exposed"
)]
fn uv_previews_all_upgrades_without_mutating_and_preserves_failures() {
    for version in ["0.4.24", "0.10.0"] {
        let dir = tempfile::tempdir().unwrap();
        let uv = dir.path().join("uv");
        let log = dir.path().join("mutations");
        let config = dir.path().join("topgrade.toml");
        fs::write(&config, "[misc]\nno_self_update = true\n").unwrap();
        fs::write(
            &uv,
            "#!/bin/sh\ncase \"$*\" in\n  --version) echo \"uv $UV_TEST_VERSION\";;\n  'self --help') exit 0;;\n  *) echo \"$*\" >> \"$UV_TEST_LOG\"; echo 'test self-update failure' >&2; exit 1;;\nesac\n",
        )
        .unwrap();
        fs::set_permissions(&uv, fs::Permissions::from_mode(0o755)).unwrap();
        let path = std::env::join_paths(
            std::iter::once(dir.path().to_path_buf()).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
        )
        .unwrap();
        for dry in [true, false] {
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_topgrade"));
            cmd.args(["--config"])
                .arg(&config)
                .args(["--only", "uv", "--no-ask-retry", "--notify-end", "never"])
                .env("PATH", &path)
                .env("TOPGRADE_SKIP_BRKC_NOTIFY", "true")
                .env("UV_TEST_VERSION", version)
                .env("UV_TEST_LOG", &log);
            if dry {
                cmd.arg("--dry-run");
            }
            let output = cmd.output().unwrap();
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if dry {
                assert!(output.status.success(), "{text}");
                assert!(text.contains("uv self update"), "{text}");
                assert!(text.contains("uv tool upgrade --all"), "{text}");
                assert_eq!(text.contains("uv python upgrade"), version == "0.10.0", "{text}");
                assert!(!log.exists(), "dry-run executed an upgrade");
            } else {
                assert!(!output.status.success(), "{text}");
                assert!(text.contains("test self-update failure"), "{text}");
                assert_eq!(fs::read_to_string(&log).unwrap(), "self update\n");
            }
        }
    }
}
