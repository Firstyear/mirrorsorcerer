# Mirror Magic tool to Magically make OpenSUSE Mirrors Magic-er

This tool will profile official instances of OpenSUSE mirrorcache
to determine the fastes repositories for your system.

    # sudo mirrormagic
    INFO mirrormagic: Mirror Magic ✨
    INFO mirrormagic: Profiling - download.opensuse.org ...
    INFO mirrormagic: Profiling - mirrorcache.opensuse.org ...
    INFO mirrormagic: Profiling - mirrorcache-au.opensuse.org ...
    INFO mirrormagic: Profiling - mirrorcache-us.opensuse.org ...
    INFO mirrormagic: Selected - https://mirrorcache-au.opensuse.org/ - time=21.891491ms
    INFO mirrormagic: do it not requested, not changing /etc/zypp/repos.d
    INFO mirrormagic: To update your mirrors re-run with '-x'

This will only update mirrors that are provided by OpenSUSE. Custom mirrors are
not altered.
