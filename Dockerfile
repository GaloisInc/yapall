FROM rust:1.68-bookworm

WORKDIR /yapall


RUN apt-get update
ARG UBUNTU_NAME="bookworm"
ARG LLVM_MAJOR_VERSION="14"
RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN echo "deb http://apt.llvm.org/${UBUNTU_NAME}/ llvm-toolchain-${UBUNTU_NAME}-${LLVM_MAJOR_VERSION} main" | tee /etc/apt/sources.list.d/llvm.list
RUN apt-get -y install --no-install-recommends llvm-${LLVM_MAJOR_VERSION} llvm-${LLVM_MAJOR_VERSION}-dev
RUN apt-get install libpolly-14-dev
RUN apt-get -y install clang-${LLVM_MAJOR_VERSION}
RUN apt-get -y install clang++-${LLVM_MAJOR_VERSION}
COPY . /yapall/
RUN cargo build --release

ENV RUST_BACKTRACE=1
CMD ["cargo", "test"]


