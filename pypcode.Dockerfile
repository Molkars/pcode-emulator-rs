FROM python:latest

RUN pip install --upgrade pip
RUN pip install pypcode

WORKDIR /home/env/

VOLUME /home/env/

ENTRYPOINT /bin/bash