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

PUBLIC_DOCS_DIR=$0
REF_NAME=$1

just build
rm -rf $PUBLIC_DOCS_DIR/api/static/$REF_NAME
mkdir -p $PUBLIC_DOCS_DIR/api/static/$REF_NAME
cp -r target/doc/* $PUBLIC_DOCS_DIR/api/static/$REF_NAME
$(cd $PUBLIC_DOCS_DIR/api && flyctl deploy --remote-only --detach)
