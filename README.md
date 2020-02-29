# Differ

![differ](./differ.jpg)

Differ is a simple tool write with Rust, It can diff two directory and generate *package.zip* and *info.json* file.

you can use it for collect upgrade file from early portable version to the latest version, just unzip *package.zip* into early version directory, then update!



## Usage

Clone

```bash
git clone https://github.com/sincerefly/differ.git
```

Run

```bash
cd differ && cargo run test01 test02
```

Output

```
...

> Diff Info

 + f3259ffb1c692d6d17b903a814b2fda6 index.js
 + a87ff679a2f3e71d9181a67b7542122c md/4.md
 + 74ead4b39e6cb4f9276ec47466a46071 images/1358088901064.jpg

> Collect

   copy __package/index.js
   copy __package/md/4.md
   copy __package/images/1358088901064.jpg

> Create Package

 adding __package/images/1358088901064.jpg as images/1358088901064.jpg
 adding __package/md/4.md as md/4.md
 adding __package/index.js as index.js
   done __package written to package.zip

time spend: 184.721768ms
Success!
```



