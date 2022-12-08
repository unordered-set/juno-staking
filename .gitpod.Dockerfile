FROM cosmwasm/cw-gitpod-base:v0.16

RUN cd /home/gitpod && \
    /usr/bin/git clone https://github.com/CosmosContracts/juno && \
    cd juno && \
    /usr/bin/git checkout v11.0.3 && \
    /usr/bin/make install