# #!/bin/bash

exit_cmd (){
  [ "$BASH_SOURCE" = "$0" ] && echo exit || echo return
}

dockpod (){
  command -v podman || echo docker
}

export CLUSTER_NAME=beavercds
COMPOSE_FILE="$(git rev-parse --show-toplevel)/tests/services.compose.yaml"

start_stuff (){
  # start cluster

  # rootless minikube has problems with extracting the cert-manager images for
  # some reason, so switch to k3d instead
  #
  # minikube start --container-runtime=cri-o

  # rootless podman? add kubelet-in-rootless arg
  if $(dockpod) info --format={{.Host.Security.Rootless}} | grep -q true ; then
    ROOTLESS_ARG='--k3s-arg=--kubelet-arg=feature-gates=KubeletInUserNamespace=true@server:*'
  else
    ROOTLESS_ARG=''
  fi

  # create cluster and expose ingress ports
  k3d cluster create "$CLUSTER_NAME" \
    -p "8000:80@loadbalancer" -p "8443:443@loadbalancer" \
    --k3s-arg "--disable=traefik@server:*" \
    --registry-config "$(git rev-parse --show-toplevel)/tests/registry.k3d.yaml" \
    "$ROOTLESS_ARG"

  # start registry
  $(dockpod) compose -f $COMPOSE_FILE up -d

  # connect registry to kube
  $(dockpod) network connect k3d-beavercds tests-registry-server-1

  # export variables if sourced or echo them if run
  export BEAVERCDS_REGISTRY_DOMAIN="localhost:5000/testing"
  export BEAVERCDS_PROFILES_TESTING_KUBECONTEXT="k3d-$CLUSTER_NAME"
  export BEAVERCDS_PROFILES_TESTING_S3_ENDPOINT="http://localhost:9000"
  export BEAVERCDS_PROFILES_TESTING_S3_REGION=""
  export BEAVERCDS_PROFILES_TESTING_S3_ACCESS_KEY=$(cat $COMPOSE_FILE | yq -r .services.minio.environment.MINIO_ROOT_USER)
  export BEAVERCDS_PROFILES_TESTING_S3_SECRET_KEY=$(cat $COMPOSE_FILE | yq -r .services.minio.environment.MINIO_ROOT_PASSWORD)

  if [ $(exit_cmd) = "exit" ] ; then
    echo
    echo "export these vars manually, or source this script to export"
    env | grep BEAVERCDS | sort
  fi
}

stop_stuff (){
  # minikube delete
  k3d cluster delete "$CLUSTER_NAME"
  $(dockpod) compose -f $(git rev-parse --show-toplevel)/tests/services.compose.yaml down --volumes
}


case "${1:-}" in
  start | up) start_stuff ;;
  stop | down | rm) stop_stuff ;;
  *)
    echo "usage:" 1>&2
    echo "  $0 up" 1>&2
    echo "  $0 down" 1>&2
    $(exit_cmd) 2
  ;;
esac
