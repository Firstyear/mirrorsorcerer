# Mirror Magic tool to Magically make OpenSUSE Mirrors Magic-er

This tool will profile official instances of OpenSUSE mirrorcache
to determine the fastes repositories for your system.

    # sudo mirrorsorcerer
    INFO mirrormagic: Mirror Sorcerer ✨
    INFO mirrormagic: Profiling - download.opensuse.org ...
    INFO mirrormagic: Profiling - mirrorcache.opensuse.org ...
    INFO mirrormagic: Profiling - mirrorcache-au.opensuse.org ...
    INFO mirrormagic: Profiling - mirrorcache-us.opensuse.org ...
    INFO mirrormagic: Selected - https://mirrorcache-au.opensuse.org/ - time=21.891491ms
    INFO mirrormagic: do it not requested, not changing /etc/zypp/repos.d
    INFO mirrormagic: To update your mirrors re-run with '-x'

This will only update mirrors that are provided by OpenSUSE. Custom mirrors are
not altered.

## Details

The primary way to use this will be to install it and allow it to run at boot

    zypper in mirrorsorcerer
    systemctl enable mirrorsorcerer
    systemctl start mirrorsorcerer

If you wish to define a custom mirror list that should be profiled instead:

    # vim /etc/my_custom_mirrors.json
    {
      "mirrors": [
        "https://download.opensuse.org",
        "..."
      ]
    }

Then you can update the unit file with:

    # systemctl edit mirrorsorcerer
    [Service]
    Environment=MIRROR_DEFS=/etc/my_custom_mirrors.json

To enable debug logging if you have an issue

    # systemctl edit mirrorsorcerer
    [Service]
    Environment=RUST_LOG=debug


