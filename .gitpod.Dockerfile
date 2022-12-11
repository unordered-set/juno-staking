FROM cosmwasm/cw-gitpod-base:v0.16

RUN cd /home/gitpod && \
    wget https://golang.org/dl/go1.19.2.linux-amd64.tar.gz && \
    tar -C /home/gitpod/go1.19.2 -xzf go1.19.2.linux-amd64.tar.gz && \
    /usr/bin/git clone https://github.com/CosmosContracts/juno && \
    cd juno && \
    /usr/bin/git checkout v11.0.3 && \
    export PATH=/home/gitpod/go1.19.2/go/bin:$PATH && \
    /usr/bin/make install