git clone https://github.com/ImageMagick/ImageMagick.git ~/ImageMagick
cd ~/ImageMagick
./configure --prefix=/usr/local
make -j$(nproc)
sudo make install
sudo ldconfig
