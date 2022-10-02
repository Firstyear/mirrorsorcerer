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
    systemctl enable --now mirrorsorcerer

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

## Undoing the changes

Mirrorsorcerer is careful to make backups before making changes.

Repo files are copied to `/etc/zypp/repos.d/*.msbak` containing their original content.
All customisations to zypp.conf are preserved when changing it. Original is backed up to /etc/zypp/zypp.conf.msbak

## Why Mirrorsorcerer - Technical Details

To understand why mirrorsorcerer works, we need to examine what zypper does in a default install
and how mirrorsorcerer alters that behaviour.

### Repository Metadata

Zypper connects to and expects to be redirected. The "primary" redirection service is based in the EU.
Two redirectors exist. download.opensuse.org (mirrorbrain) and mirrorcache.opensuse.org (mirrorcache).
mirrorbrain will return metalink file, mirrorcache returns http redirects.

Due to download.opensuse.org *and* mirrorcache.opensuse.org being in EU, the latency is in the order
of 350ms for each Round Trip. This quickly adds up, where a single HTTP GET request can take approximately
1 second from my home internet connection (Australia 20ms to my ISP).

There are 4 required files for one repository (repomd.xml, media, repomd.xml.key and repomd.xml.asc).
zypper initially performs a HEAD request for repomd.xml and then *closes the connection*. If this is
considered "out of date", zypper then opens a second connection and requests the full set of 4 files.

From my connection the HEAD request takes 0.7s. The second series of GET requests take 2.6s from
first connection open to closing.

If we are to perform a full refresh this process of double connecting repeats for each repository we
have, taking ~3.2s just in network operations.

Given an opensuse/tumbleweed:latest container, and running `time zypper ref --force` takes 32 seconds
to complete (2022-10-02) . The addition of further repositories linearly increases this time taken.

Zypper also aggresively refreshes metadata. By default metadata is considered out of date after 10
minutes. The most common user perception of this is that zypper after a small period of inactivity
will then have a ~30 second delay before responding on the next innvocation.

### Package Downloads (mirrorcache)

Let's assume we have our opensuse/tumbleweed:latest container, and we are running "zypper in -y less" (2022-10-02). This should
result in the need to download 9 rpms:   busybox busybox-which file file-magic less libcrypt1 libmagic1 libseccomp2 libsepol2

zypper starts by sending an initial GET request to download.opensuse.org for `/tumbleweed/repo/oss/media.1/media`
which returns a 200 and the name of the current media build.

zypper then requests `/tumbleweed/repo/oss/noarch/file-magic-5.43-1.1.noarch.rpm`. The response is a
HTTP 302 to the australian mirrorcache instance mirrocache-au.opensuse.org.

`file-magic-5.43-1.1.noarch.rpm` is now requested from mirrorcache-au.opensuse.org, and a metalink
xml is returned:

    <metalink xmlns="urn:ietf:params:xml:ns:metalink">
      <generator>MirrorCache</generator>
      <origin dynamic="true">http://mirrorcache-au.opensuse.org/tumbleweed/repo/oss/noarch/file-magic-5.43-1.1.noarch.rpm</origin>
      <published>2022-10-02T12:01:37Z</published>
      <publisher>
        <name>openSUSE</name>
        <url>http://download.opensuse.org</url>
      </publisher>
      <file name="file-magic-5.43-1.1.noarch.rpm">
        <!-- Mirrors which handle this country (AU):  -->
        <url location="AU" priority="1">http://mirror.firstyear.id.au/tumbleweed/repo/oss/noarch/file-magic-5.43-1.1.noarch.rpm</url>
        <!-- Mirrors in the same continent (OC):  -->
        <url location="NZ" priority="2">http://mirror.2degrees.nz/opensuse/tumbleweed/repo/oss/noarch/file-magic-5.43-1.1.noarch.rpm</url>
        <!-- Mirrors in other parts of the world:  -->
        <!-- File origin location:  -->
        <url location="" priority="3">http://mirror.firstyear.id.au/tumbleweed/repo/oss/noarch/file-magic-5.43-1.1.noarch.rpm</url>
      </file>
    </metalink>

It is not always clear the selection logic that is used by zypper to decide between mirrors in a metalink xml.

Finally zypper now connects to mirror.firstyear.id.au and retrieves the file.

What is interesting in this process is:

* The connection to mirrorcache-au.opensuse.org is always closed after the 302.
* The connections to download.opensuse.org and mirror.firstyear.id.au are pooled (hooray!)

Now using this information we can determine the impact of *latency* in these requests.

Each 302 from download.opensuse.org takes 0.35 seconds to resolve.

The lack of re-use on the mirrorcache-au connection adds 0.03 seconds for each file request due to
having to re-open the connection. This whole connection takes 0.16 seconds to complete.

From this, of the total 11.75 seconds to complete the install, 3.15 seconds are just in requests
to download.opensuse.org, and the lack of connection reuse to the local redirector adds 0.27 seconds
to the operation.

### Changes that mirrorsorcerer makes

Mirrorsorcerer makes two key changes on your system to improve this situation.

* Repository metadata timeout is set to 18 hours from 10 minutes
* Replacement of download.opensuse.org with a lower-latency mirror

The increase in metadata timeout means that zypper will only refresh metadata once a day. Given that
tumbleweed snapshots are "daily", and users with custom OBS repos and development work will refresh
manually, this "delay" generally causes no difference in user experience.

By directing zypper to a local redirector instead of going through download.opensuse.org we reduce
the major source of latency, and prevent the connection open/close issue on the intermediate 302 host.
In theory from our former experiment this should reduce the install from 11.75 seconds to 8.6 seconds
however in reality this change is actually far better. The install time is reduced to 6.6 seconds.
That is a saving of 5.15 seconds, 44% of the original execution time.

### Issues mirrorsorcerer can NOT prevent

The primary remaining issues that mirrorsorcerer can not prevent that causes reduction in bandwidth
is *range requests*.

*You can work around this today by setting ZYPP_MULTICURL=0 in your environment*

In some situations, zypper will attempt to "strip" downloads with range requests over multiple mirrors.
For example, when retrieving libgio, we can see the metalink xml:

    GET /distribution/leap/15.4/repo/oss/x86_64/libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm HTTP/1.1
    Host: mirrorcache-au.opensuse.org
    User-Agent: ZYpp 17.30.2 (curl 7.79.1) 
    X-ZYpp-AnonymousId: 8daf9dba-ee79-4878-a8c1-9c41a5d70390
    X-ZYpp-DistributionFlavor: appliance-docker
    Accept: */*, application/metalink+xml, application/metalink4+xml

    HTTP/1.1 200 OK
    content-disposition: attachment; filename="libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm.meta4"
    content-length: 1819
    content-type: application/metalink4+xml; charset=UTF-8
    date: Sun, 02 Oct 2022 03:06:09 GMT
    server: Mojolicious (Perl)
    vary: Accept-Encoding
    connection: close

    <?xml version="1.0" encoding="UTF-8"?>

    <metalink xmlns="urn:ietf:params:xml:ns:metalink">
      <generator>MirrorCache</generator>
      <origin dynamic="true">http://mirrorcache-au.opensuse.org/distribution/leap/15.4/repo/oss/x86_64/libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm</origin>
      <published>2022-10-02T13:06:09Z</published>
      <publisher>
        <name>openSUSE</name>
        <url>http://download.opensuse.org</url>
      </publisher>
      <file name="libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm">
        <!-- Mirrors which handle this country (AU):  -->
        <url location="AU" priority="1">http://mirror.aarnet.edu.au/pub/opensuse/opensuse/distribution/leap/15.4/repo/oss/x86_64/libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm</url>
        <url location="AU" priority="2">http://ftp.iinet.net.au/pub/opensuse/distribution/leap/15.4/repo/oss/x86_64/libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm</url>
        <url location="AU" priority="3">http://mirror.firstyear.id.au/distribution/leap/15.4/repo/oss/x86_64/libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm</url>
        <url location="AU" priority="4">http://ftp.netspace.net.au/pub/opensuse/distribution/leap/15.4/repo/oss/x86_64/libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm</url>
        <url location="AU" priority="5">http://mirror.internode.on.net/pub/opensuse/distribution/leap/15.4/repo/oss/x86_64/libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm</url>
        <!-- Mirrors in the same continent (OC):  -->
        <url location="NZ" priority="6">http://mirror.2degrees.nz/opensuse/distribution/leap/15.4/repo/oss/x86_64/libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm</url>
        <!-- Mirrors in other parts of the world:  -->
        <!-- File origin location:  -->
        <url location="" priority="7">http://mirror.firstyear.id.au/distribution/leap/15.4/repo/oss/x86_64/libgio-2_0-0-2.70.4-150400.1.5.x86_64.rpm</url>
      </file>
    </metalink>

We can then see then that zypper attempts to strip this over 5 mirrors:

* Range: bytes=0-131071 - mirror.aarnet.edu.au
* Range: bytes=131072-262143 - ftp.iinet.net.au
* Range: bytes=393216-524287 - ftp.netspace.net.au
* Range: bytes=524288-655359 - mirror.internode.on.net
* Range: bytes=655360-702155 - mirror.2degrees.nz

From the start to the end of these requests completing, this takes 0.32 seconds to download 702155 bytes. Now compare, if we download directly
from a single mirror, this file downloads in 0.19 seconds.

On smaller files this range behaviour is not "so bad", as there is obviously a difference in performance, but only ~30%. On the lowest priority
mirror, the single download takes "as long" as the striped ranges (0.31 seconds).

On larger files however
we see this have a much larger impact. `rust1.63-1.63.0-150300.7.3.1.x86_64.rpm` for example takes 29.06 seconds to retrieve 83951816 bytes. However
a direct connection to the preferred mirror take 9.7 seconds. That is 1/3rd of the time required. Even to the "lowest" priority mirror (2degrees.nz) this download
takes 22 seconds directly. This means that by *not* using range requests, zypper will range from 25% to 78% faster. There appears to be no situation where
range requests are *faster* than directly connecting to a single mirror. It's likely this is due to:

* Small range requests can not reach maximum speed due to 'bursty' behaviour.
* Mirror storage tends to be optimised to streaming reads not random IOPS.

In addition there is actually a *bug* in zypper where if any mirror responds with a 200, instead of
consuming the entire file from that mirror and ceasing range requests, it will continue to issue range
requests to that mirror and other mirrors instead.

Zypper could resolve this issue by:

* Remove range requests outright. If "parallel" downloads over multiple files becomes a feature, why multiplex that further?
* Increasing the range chunk size to allow connections to read larger throughputs. For example if a file is 80M then request 5 times 20M chunks rather than 640 times 131072 byte chunks. This will allow connections to reach better throughputs.
* Disable range requests on files smaller than 4Mb.

