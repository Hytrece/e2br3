#!/usr/bin/env sh
set -eu

cat >/etc/systemd/system/e2br3-dev-update.service <<'EOF'
[Unit]
Description=Update E2BR3 dev containers
After=docker.service network-online.target

[Service]
Type=oneshot
ExecStart=/opt/e2br3/deploy/ec2/update-dev.sh
EOF

cat >/etc/systemd/system/e2br3-dev-update.timer <<'EOF'
[Unit]
Description=Check E2BR3 dev images every five minutes

[Timer]
OnBootSec=1min
OnUnitActiveSec=5min

[Install]
WantedBy=timers.target
EOF

systemctl daemon-reload
systemctl enable --now e2br3-dev-update.timer
systemctl start e2br3-dev-update.service
