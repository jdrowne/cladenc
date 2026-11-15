#!/bin/sh
printf '%s\n' "$@" > arguments
echo "$CLADENC_MODEL $CLADENC_MODEL_URL ${CLADENC_MODEL_KEY-none}" > model
