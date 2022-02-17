# Mirror Magic tool to Magically make OpenSUSE Mirrors Magic-er

This tool will profile official instances of OpenSUSE mirrorcache
to determine the fastest repositories for your system.

    # sudo mirrorsorcerer
    INFO ✨ Mirror Sorcerer ✨
    INFO Profiling - mirrorcache.opensuse.org - 2001:67c:2178:8::16 - insufficient data
    INFO Profiling - mirrorcache.opensuse.org - 195.135.221.140 - time=320.972483ms
    INFO Profiling - mirrorcache-au.opensuse.org - 2400:8907::f03c:92ff:fe82:7bb - insufficient data
    INFO Profiling - mirrorcache-au.opensuse.org - 172.105.167.90 - time=25.093516ms
    INFO Profiling - mirrorcache-us.opensuse.org - 2a07:de40:401::65 - insufficient data
    INFO Profiling - mirrorcache-us.opensuse.org - 91.193.113.65 - time=197.407333ms
    INFO Selected - https://mirrorcache-au.opensuse.org/ - time=25.093516ms
    INFO 🪄  updating repo repo-oss -> https://mirrorcache-au.opensuse.org/ports/aarch64/tumbleweed/repo/oss/
    INFO 🪄  updating repo repo-debug -> https://mirrorcache-au.opensuse.org/ports/aarch64/debug/tumbleweed/repo/oss/
    INFO 🪄  updating repo repo-source -> https://mirrorcache-au.opensuse.org/ports/aarch64/source/tumbleweed/repo/oss/
    INFO 🪄  updating repo repo-update -> https://mirrorcache-au.opensuse.org/ports/aarch64/update/tumbleweed/
    INFO 🔮 watching /etc/zypp/repos.d for changes ...
    INFO 🪄  updating repo network:idm -> https://mirrorcache-au.opensuse.org/repositories/network:/idm/openSUSE_Tumbleweed


This will only update mirrors that are provided by OpenSUSE. Custom mirrors are
not altered. If you add new repositories, they are dynamically updated.

## Details

The primary way to use this will be to install it and allow it to run at boot

    zypper in mirrorsorcerer
    systemctl enable mirrorsorcerer
    systemctl start mirrorsorcerer

If you wish to define a custom mirror list that should be profiled instead:

    # vim /etc/my_custom_mirrors.json
    {
      "replaceable": [
        "https://download.opensuse.org"
      ],
      "mirrors": [
        "..."
      ]
    }

* `mirrors` - The list of mirrors in the "pool" that you want to profile and potentially select from.
* `replaceable` - A list of mirrors that are "excluded" from the pool, but could be replaced with a pooled mirror.

Then you can update the unit file with:

    # systemctl edit mirrorsorcerer
    [Service]
    Environment=MIRROR_DEFS=/etc/my_custom_mirrors.json

To enable debug logging if you have an issue

    # systemctl edit mirrorsorcerer
    [Service]
    Environment=RUST_LOG=debug



