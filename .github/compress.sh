apt-get install zip
cd packages
for files in $(ls)
do
  zip $files.zip $files
done
for files in $(ls *.exe.zip)
do
  mv $files ${files%%.*}.zip
done
