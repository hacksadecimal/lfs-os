# LFS Omni Store

Use a cloud object store or a local file share as the backing storage for git-lfs.

### Installation

```sh
cargo install --force lfs-os
```

### Usage

Configuration of lfs-os is mostly configuration of git-lfs, which is itself configured through
the git config system. As such, you can configure lfs-os globally, for some specific paths,
or on a per repository basis. The following examples show configuring a specific repository. They assumes that you are in the root of the repo you would like to configure and lfs-os is installed and on your path.


#### Example: Locally mounted NFS/SMB/SSHFS

```sh
git lfs install --local
git config --add lfs.customtransfer.lfs-os.path "$(which lfs-os)"
git config --add lfs.standalonetransferagent "lfs-os"
git config --add lfs.customtransfer.lfs-os.args "--provider local --uri <path-to-mount>"
```

#### Example: AWS S3

```sh
git lfs install --local
git config --add lfs.customtransfer.lfs-os.path "$(which lfs-os)"
git config --add lfs.standalonetransferagent "lfs-os"
git config --add lfs.customtransfer.lfs-os.args "--provider AWS --uri <s3 bucket url>"
```

For detailed information on what options you can configure for `lfs.customtransfer.lfs-os.args`, view the help by running `lfs-os --help`. Keep in mind, the lfs-os binary will be invoked by `git-lfs`, except for viewing the help, you never run `lfs-os` manually.

## Acknowledgments:
This project was created after reading through [Nicolas Graves' lfs-s3](https://github.com/nicolas-graves/lfs-s3) which is why the protocol structs are divided up the same way. Transitively, this project wouldn`t exist in the same form without [Sinbad's lfs-folderstore](https://github.com/sinbad/lfs-folderstore).

## License

SPDX-License-Identifier: MIT OR Apache-2.0

This project is dual licensed by MIT and Apache-2.0.
