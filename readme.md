# bees
    bees是基于axum架构，以及服务转发的模式实现多级平台直接服务的转发

# 适用场景
    当网络拓扑为A->B->C,且A到C直接没有直达的网络，需要B来做转发的时候，就可以利用此服务


# 部署
    配置文件为bees.yaml


# 打包
    cargo build --release