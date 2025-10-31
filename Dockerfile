FROM infinilabs/baseos:24
ADD pkg /pkg
RUN cd /pkg && ./extract-cli hello.pdf > out
