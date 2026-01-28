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

PUBLIC_DOCS_DIR=$1
REF_NAME=$2

cargo d --no-deps --workspace --all-features --target-dir ./target --exclude tests --exclude examples
echo "<meta http-equiv=\"refresh\" content=\"0; url=substrate\">" > ./target/doc/index.html
rm -rf $PUBLIC_DOCS_DIR/api/static/$REF_NAME
mkdir -p $PUBLIC_DOCS_DIR/api/static/$REF_NAME
cp -r ./target/doc/. $PUBLIC_DOCS_DIR/api/static/$REF_NAME
cd $PUBLIC_DOCS_DIR/api
