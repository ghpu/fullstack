FROM rust
LABEL version="v1" \
      maintainer="ghislain.putois@orange.com" \
      name="Case analyzer" \
      licence="MIT" \
      info="NLP Evaluation suite"

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y rollup curl && \
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh 

COPY Makefile /code/Makefile
COPY common /code/common
COPY frontend /code/frontend
COPY backend /code/backend

VOLUME /code
WORKDIR /code
ENTRYPOINT ["make"]

