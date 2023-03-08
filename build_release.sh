version=$(cargo pkgid | grep -o '#.*' | cut -c2-10)
git stash
git tag "$version"
cargo build
for arch in "x86_64-pc-windows-gnu" "x86_64-unknown-linux-gnu"
do
cargo build --release --target $arch
mkdir -p ./releases/$version/$arch
cp ./target/$arch/release/airlog* ./releases/$version/$arch
rm ./releases/$version/$arch/airlog.d
done
git add *
git commit -m "Create release $version"
git push
