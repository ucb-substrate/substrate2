# Print commands that are executed
set -x

# Turn on error checking options
# Running the bashrc can result in errors,
# which we'll just ignore.
set -euf -o pipefail

if [ $# -eq 0 ]
  then
    echo "Usage: ./deploy.sh PUBLIC_DOCS_DIR REF_NAME"
    exit 1
fi

PUBLIC_DOCS_DIR=$1
REF_NAME=$2

cat << EOF > ./site-config.json
{
    "branch": "$REF_NAME"
}
EOF
yarn install --frozen-lockfile
yarn build
if [ $REF_NAME = "main" ]; then
    find $PUBLIC_DOCS_DIR/docusaurus/static -not -path "$PUBLIC_DOCS_DIR/docusaurus/static/branch/*" -not -name "fly.toml" -not -name "Dockerfile" -delete
    mkdir -p $PUBLIC_DOCS_DIR/docusaurus/static
    cp -r ./build/. $PUBLIC_DOCS_DIR/docusaurus/static
else
    rm -rf $PUBLIC_DOCS_DIR/docusaurus/static/branch/$REF_NAME
    mkdir -p $PUBLIC_DOCS_DIR/docusaurus/static/branch/$REF_NAME
    cp -r ./build/. $PUBLIC_DOCS_DIR/docusaurus/static/branch/$REF_NAME
fi
cd $PUBLIC_DOCS_DIR/docusaurus
flyctl deploy --remote-only --detach
