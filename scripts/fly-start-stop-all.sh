#!/bin/bash
# Start or stop all machines in all Fly.io apps in a given organization
# Usage: ./stop_all_fly_machines.sh <start|stop> <org-name>

set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <start|stop> <org-name>" >&2
  exit 1
fi

ACTION="$1"
ORG="$2"

if [[ "$ACTION" != "start" && "$ACTION" != "stop" ]]; then
  echo "First argument must be 'start' or 'stop'" >&2
  exit 1
fi

APPS=$(fly apps list --org "$ORG" --json | jq -r '.[].Name')

if [ -z "$APPS" ]; then
  echo "No apps found in org $ORG" >&2
  exit 0
fi


for APP in $APPS; do
  echo "${ACTION}ing all machines in app: $APP"
  MACHINE_IDS=$(fly machines list -a "$APP" --json | jq -r '.[].id')
  if [ -z "$MACHINE_IDS" ]; then
    echo "  No machines found in $APP"
    continue
  fi
  for ID in $MACHINE_IDS; do
    echo "  ${ACTION}ing machine $ID in $APP"
    fly machines $ACTION "$ID" -a "$APP" || echo "    Failed to $ACTION machine $ID in $APP"
  done
done

echo "Done."
