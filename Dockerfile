FROM ubuntu:20.04

RUN apt-get update -yqq \
    && apt-get install -y --no-install-recommends \
               wget \
               ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    ;

RUN wget https://git.io/teip-1.2.0.x86_64.deb && \
    dpkg -i ./teip*.deb

ENTRYPOINT ["teip"]
CMD ["--help"]
