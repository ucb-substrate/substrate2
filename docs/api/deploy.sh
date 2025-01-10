# Print commands that are executed
set -x

# Turn on error checking options
# Running the bashrc can result in errors,
# which we'll just ignore.
set -euf -o pipefail

if [ $# -le 1 ]; then
    echo "Usage: ./deploy.sh PUBLIC_DOCS_DIR REF_NAME"
    exit 1
fi

just build
rm -rf $0/api/static/$REF_NAME
mkdir -p $0/api/static/$REF_NAME
cp -r target/doc/* $0/api/static/$REF_NAME
$(cd $0/api && flyctl deploy --remote-only --detach)
