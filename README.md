# vultr_ddns
学习rust写的一个简单程序

# 使用方法

## Docker

1.安装docker `curl -fsSL get.docker.com | bash`

2.运行 `docker run --name <容器名字> --restart always -d evlan/vultr_ddns:latest --domain-head <域名头head> --domain-body <域名body> --api-key <vultr的API_KEY> --check-ip-url <获取ip地址url> --sleep-sec <检测IP间隔,默认60秒>`

> 例子 `docker run --name ddns --restart always -d evlan/vultr_ddns:latest --domain-head ddns.hk --domain-body abc.com --api-key xxxxxxxxxxxxxx --check-ip-url https://ip.42.pl/short`

> 查看最新20条日志 `docker logs -f -n 20 ddns`

## rust编译

1.安装rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

3.克隆仓库并进入目录 `git clone https://github.com/yi-shiyu/vultr_ddns.git && cd vultr_ddns`

4.编译 `cargo build --release`

5.移动到bin目录 `cp target/release/vultr_ddns /usr/bin/`

6.执行 `nohup vultr_ddns --domain-head <域名头head> --domain-body <域名body> --api-key <vultr的API_KEY> --check-ip-url <获取ip地址url> --sleep-sec <检测IP间隔,默认60秒>  >> /root/ddns.log &`

> 例子 `nohup vultr_ddns --domain-head ddns.hk --domain-body abc.com --api-key xxxxxxxxxxxxxx --check-ip-url https://ip.42.pl/short >> /root/ddns.log &`

> 查看最新20条日志 `tail -f -n 20 /root/ddns.log`
