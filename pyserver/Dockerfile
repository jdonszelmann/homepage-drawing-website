FROM python:3.7-alpine
RUN pip3 install websockets asyncio
COPY . ./
EXPOSE 80

CMD ["python3","main.py"]
