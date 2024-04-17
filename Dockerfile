FROM rust:1.77.1 as builder


RUN echo 'Asia/Shanghai' >/etc/timezone && \
    ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime



ENV TZ=Asia/Shanghai LANG=C.UTF-8


ADD . /bees

WORKDIR /bees


RUN cargo build
# RUN cargo build --release
## 多阶段编译
FROM rust:1.77.1


WORKDIR /app/bees
#RUN mkdir -p "/usr/lib/x86_64-linux-gnu/"
#COPY --from=builder /usr/lib/x86_64-linux-gnu/libssl*  /usr/lib/x86_64-linux-gnu/
#COPY --from=builder /usr/lib/x86_64-linux-gnu/libcrypt*  /usr/lib/x86_64-linux-gnu/
COPY --from=builder /bees/target/debug/bees /app/bees/bees
# 声明服务端口
EXPOSE 7601
ENTRYPOINT [ "/app/bees/bees" ]
