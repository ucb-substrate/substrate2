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

echo << EOF
{
    "branch": "$REF_NAME"
    "edit_url": "https://github.com/substrate-labs/substrate2/tree/$REF_NAME/docs/docusaurus"
}
EOF
yarn install
yarn build
if [ $1 -eq "main" ]; then
    find $0/docusaurus/static -not -path "$0/docusaurus/static/branch/*" -not -name "fly.toml" -not -name "Dockerfile" -delete
    mkdir -p $0/docusaurus/static
    cp -r build/* $0/docusaurus/static/branch/$REF_NAME
else
    rm -rf $0/docusaurus/static/branch/$REF_NAME
    mkdir -p $0/docusaurus/static/branch/$REF_NAME
    cp -r build/* $0/docusaurus/static/branch/$REF_NAME
fi
$(cd $0/docusaurus && flyctl deploy --remote-only --detach)
