#!/bin/bash
set -ex

id="${RANDOM}"
disk_name="archlinux-builder-data-${id}"
instance_name="archlinux-builder-${id}"
zone='asia-northeast1-a'
machine_type='n1-standard-1'
image_name="archlinux-$(date +%Y-%m-%d)-${id}"

# user defined
network='gcp'
subnet='apne1'

echo "Create disk $disk_name"
gcloud compute disks create "$disk_name" \
  --type pd-ssd \
  --size 10GB \
  --zone "$zone"

echo "Create instance $instance_name"
gcloud compute instances create "$instance_name" \
  --image-project ubuntu-os-cloud \
  --image-family ubuntu-1604-lts \
  --machine-type "$machine_type" \
  --zone "$zone" \
  --network "$network" \
  --subnet "$subnet" \
  --metadata-from-file startup-script=startup-script.sh \
  --scopes compute-rw \
  --disk "name=$disk_name,mode=rw"

cat <<EOS > create-image-next-${id}.sh
#!/bin/bash
set -ex

echo "Delete instance $instance_name"
gcloud compute instances delete "$instance_name" \
  --quiet \
  --zone "$zone"

echo "Create image $image_name from $disk_name"
gcloud compute images create "$image_name" \
  --family archlinux \
  --source-disk "$disk_name" \
  --source-disk-zone "$zone"

echo "Delete disk $disk_name"
gcloud compute disks delete "$disk_name" \
  --quiet \
  --zone "$zone"

EOS
chmod +x create-image-next-${id}.sh
echo "Run create-image-next-${id}.sh after startup-script.sh finished"
