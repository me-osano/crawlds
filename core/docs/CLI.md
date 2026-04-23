# CLI Reference

All commands accept `--json` / `-j` for raw JSON output (useful in scripts).
All commands accept `--socket <path>` to override the daemon socket.

---

## run

```bash
crawlds run                          # run quickshell interactively
crawlds run -c ~/.config/qs/my       # use custom config
crawlds run -d                       # run as daemon
```

## restart

```bash
crawlds restart                      # restart running shell
```

## kill

```bash
crawlds kill                         # kill running shell
```

## ipc

```bash
crawlds ipc                          # list IPC targets
crawlds ipc <target> <func> [args]   # call IPC function
```

## update

```bash
crawlds update                       # update to latest
crawlds update --dry-run            # check for updates
```

## version

```bash
crawlds version                      # show version
crawlds version -j                   # JSON output
```

---

## brightness

```bash
crawlds brightness                    # get current
crawlds brightness --set=80           # set to 80%
crawlds brightness --inc=5            # increase by 5%
crawlds brightness --dec=10           # decrease by 10%
```

## sysmon

```bash
crawlds sysmon --cpu                  # CPU usage + load averages
crawlds sysmon --mem                  # memory usage
crawlds sysmon --disk                 # disk usage per mount
crawlds sysmon --watch                # live CPU/memory updates (SSE)
crawlds sysmon --cpu --json           # raw JSON
```

## bluetooth

```bash
crawlds bluetooth                            # status + device list
crawlds bluetooth --scan                     # start discovery
crawlds bluetooth --connect=AA:BB:CC:DD:EE:FF
crawlds bluetooth --disconnect=AA:BB:CC:DD:EE:FF
crawlds bluetooth --power=on
crawlds bluetooth --power=off
```

## network

```bash
crawlds network                                # connectivity status
crawlds network --power=on                     # enable networking
crawlds network --power=off                    # disable networking

crawlds network --wifi --list                  # list nearby WiFi networks
crawlds network --wifi --details               # active Wi-Fi details
crawlds network --wifi --scan                  # trigger WiFi scan
crawlds network --wifi --connect --ssid=MySSID --password=hunter2
crawlds network --wifi --disconnect
crawlds network --wifi --forget --ssid=MySSID # remove saved Wi-Fi profile

crawlds network --eth --list                  # list wired interfaces
crawlds network --eth --details --iface=enp3s0 # ethernet details
crawlds network --eth --connect                # connect first wired interface
crawlds network --eth --connect --iface=enp3s0 # connect specific wired interface
crawlds network --eth --disconnect            # disconnect active wired interface
crawlds network --eth --disconnect --iface=enp3s0

crawlds network --hotspot                       # hotspot status
crawlds network --hotspot --connect            # start hotspot (defaults: ssid=CrawlDS-Hotspot)
crawlds network --hotspot --connect --ssid=MyHotspot --password=hunter2 --band=5GHz --backend=hostapd
crawlds network --hotspot --disconnect         # stop hotspot
```

## power

```bash
crawlds power                         # battery percent, state, time estimates
crawlds power --json
```

## notify

```bash
crawlds notify --list                 # all active notifications
crawlds notify --title="Build done" --body="cargo build succeeded"
crawlds notify --title="Alert" --body="Disk full" --urgency=critical
crawlds notify --dismiss=42           # dismiss notification by ID
```

## clipboard

```bash
crawlds clipboard --get                    # current clipboard content
crawlds clipboard --set="some text"        # write to clipboard
crawlds clipboard --history                # clipboard history (JSON)
```

## proc

```bash
crawlds proc                          # top 20 processes by CPU
crawlds proc --sort=mem --top=10      # top 10 by memory
crawlds proc --find=firefox           # find by name
crawlds proc --kill=1234              # SIGTERM
crawlds proc --kill=1234 --force      # SIGKILL
crawlds proc --watch=1234             # wait for PID to exit
```

## disk

```bash
crawlds disk                          # list block devices
crawlds disk --mount=/dev/sdb1        # mount device
crawlds disk --unmount=/dev/sdb1
crawlds disk --eject=/dev/sdb         # eject drive
```

## daemon

```bash
crawlds daemon                        # status + version
crawlds daemon --restart
crawlds daemon --stop
```
