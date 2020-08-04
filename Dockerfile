FROM rust AS builder
LABEL version="v1" \
      maintainer="ghislain.putois@orange.com" \
      name="Case analyzer Builder" \
      licence="MIT" \
      info="NLP Evaluation suite builder"

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y npm && \
    npm install -g rollup && \
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh 

COPY Makefile /code/Makefile
COPY common /code/common
COPY frontend /code/frontend
COPY backend /code/backend

RUN cd /code && make

FROM debian
LABEL version="v1" \
      maintainer="ghislain.putois@orange.com" \
      name="Case analyzer" \
      licence="MIT" \
      info="NLP Evaluation suite"

RUN apt-get update && \
    apt-get upgrade -y

EXPOSE 8080
VOLUME /code
WORKDIR /code
COPY --from=builder /code/out /code/out
CMD ["./out/server"]
