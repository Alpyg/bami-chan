deploy node:
  -docker service rm bami
  docker build -t bami .
  docker push 10.147.20.18:10000/bami
  docker service create -d --env-file .env \
    --constraint "node.hostname == {{node}}" \
    --name bami 10.147.20.18:10000/bami:latest
