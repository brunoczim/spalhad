#!/usr/bin/env bash

set -e

bind_args=("--bind" "0.0.0.0:5000")

kv_channel_size_args=()
if [ -n "${SPALHAD_KV_CHANNEL_SIZE}" ]
then
    kv_channel_size_args=("--kv-channel-size" "${SPALHAD_KV_CHANNEL_SIZE}")
fi

persistence_dir_args=()
if ([ -n "${SPALHAD_KV_DIR}" ] \
    && [ "${SPALHAD_KV_DIR}" != 0 ] \
    && [ "${SPALHAD_KV_DIR}" != false ] \
    && [ "${SPALHAD_KV_DIR}" != False ] \
    && [ "${SPALHAD_KV_DIR}" != FALSE ] \
    && [ "${SPALHAD_KV_DIR}" != f ] \
    && [ "${SPALHAD_KV_DIR}" != F ] \
    && [ "${SPALHAD_KV_DIR}" != nil ] \
    && [ "${SPALHAD_KV_DIR}" != Nil ] \
    && [ "${SPALHAD_KV_DIR}" != NIL ] \
    && [ "${SPALHAD_KV_DIR}" != null ] \
    && [ "${SPALHAD_KV_DIR}" != Null ] \
    && [ "${SPALHAD_KV_DIR}" != NULL ] \
    && [ "${SPALHAD_KV_DIR}" != none ] \
    && [ "${SPALHAD_KV_DIR}" != None ] \
    && [ "${SPALHAD_KV_DIR}" != None ])
then
    persistence_dir="kv"
    mkdir -p "${persistence_dir}"
    persistence_dir_args=("--persistence-dir" "${persistence_dir}")
fi

cluster_config_args=()
if [ -n "${SPALHAD_CLUSTER_CONFIG}" ]
then
    cluster_config_args=("--cluster-config" "${SPALHAD_CLUSTER_CONFIG}")
fi

if [ -z "${SPALHAD_SELF_ID}" ]
then
    echo 2>&1 "SPALHAD_SELF_ID must be defined and non-empty"
    exit 1
fi
self_id_args=("--self-id" "${SPALHAD_SELF_ID}")

exec ./server \
    "${bind_args[@]}" \
    "${kv_channel_size_args[@]}" \
    "${persistence_dir_args[@]}" \
    "${cluster_config_args[@]}" \
    "${self_id_args[@]}"
