# Analisis Benchmark Block Cipher

## Ringkasan hasil

Berdasarkan pengujian benchmark pada mode `CBC`, `CFB`, `OFB`, dan `CTR`, bentuk grafik yang dihasilkan tergolong normal untuk implementasi block cipher edukatif yang dibuat dalam bahasa C. Pengujian dilakukan pada beberapa ukuran data, mulai dari `1 KiB` hingga `4 MiB`, dengan tiga kali pengulangan pada setiap ukuran agar dapat dihitung rata-rata dan deviasi standarnya.

Secara umum terlihat bahwa latensi meningkat seiring bertambahnya ukuran data, sedangkan throughput cenderung stabil pada ukuran kecil hingga menengah, lalu sedikit menurun pada ukuran data yang lebih besar. Pola seperti ini merupakan perilaku yang wajar karena semakin banyak blok yang diproses, semakin besar pula waktu total yang dibutuhkan untuk enkripsi maupun dekripsi.

## Interpretasi grafik

### 1. Latensi meningkat saat ukuran data bertambah

Pada grafik `Latensi Enkripsi` dan `Latensi Dekripsi`, nilai waktu per iterasi naik cukup tajam saat ukuran data berubah dari `256 KiB` ke `1 MiB`, lalu meningkat lebih besar lagi pada `4 MiB`. Hal ini normal karena implementasi cipher memproses data per blok 64-bit secara berulang, sehingga semakin besar data masukan maka semakin banyak ronde total yang dijalankan.

### 2. Throughput relatif stabil lalu menurun

Pada grafik `Throughput Enkripsi` dan `Throughput Dekripsi`, nilai throughput untuk ukuran kecil dan menengah masih berada pada kisaran yang relatif dekat. Setelah ukuran data semakin besar, throughput mulai turun. Ini juga normal karena pada ukuran besar, biaya akses memori, penyalinan buffer, dan alokasi dinamis mulai lebih berpengaruh terhadap waktu total eksekusi.

### 3. Ukuran data kecil lebih berisik

Pada ukuran seperti `1 KiB`, hasil benchmark bisa sedikit lebih fluktuatif dibanding ukuran yang lebih besar. Penyebabnya adalah overhead non-kriptografi seperti pemanggilan fungsi, pengukuran waktu, alokasi memori, dan aktivitas sistem operasi menjadi cukup dominan ketika data yang diproses masih sangat kecil.

### 4. Dekripsi dapat terlihat lebih cepat dari enkripsi

Pada beberapa mode, throughput dekripsi terlihat sedikit lebih tinggi daripada throughput enkripsi. Hal ini masih wajar pada implementasi ini karena benchmark mengukur keseluruhan pipeline fungsi, bukan hanya inti transformasi block cipher. Di dalam proses enkripsi dan dekripsi terdapat overhead tambahan seperti:

- pembangkitan round key
- alokasi dan dealokasi buffer
- penyalinan data
- padding dan unpadding khusus untuk mode `CBC`

Karena itu, perbedaan kecil antara enkripsi dan dekripsi tidak selalu berarti algoritma dekripsi secara matematis lebih sederhana, tetapi dapat dipengaruhi oleh detail implementasi program.

## Penyebab utama bentuk kurva benchmark

Bentuk kurva pada grafik dipengaruhi oleh beberapa faktor utama berikut:

1. **Jumlah ronde tetap untuk setiap blok**  
   Setiap blok diproses melalui 8 ronde Feistel, sehingga total waktu eksekusi akan meningkat linear terhadap jumlah blok data.

2. **Round function cukup padat operasi**  
   Pada setiap ronde, program melakukan kombinasi operasi `XOR`, substitusi `S-Box`, dan rotasi/permutasi bit. Walaupun masing-masing operasi ringan, akumulasi pada banyak blok membuat waktu proses meningkat signifikan.

3. **Mode operasi menambah overhead**  
   `CBC`, `CFB`, `OFB`, dan `CTR` masing-masing membutuhkan mekanisme chaining, feedback, atau counter. Mekanisme ini menambah kerja per blok di luar inti block cipher.

4. **Alokasi dan copy buffer ikut terukur**  
   Benchmark ini mengukur fungsi enkripsi/dekripsi lengkap, sehingga biaya `malloc`, `free`, cloning buffer, dan padding juga ikut tercatat dalam hasil benchmark.

5. **Efek cache dan memori pada data besar**  
   Ketika ukuran data makin besar, beban akses memori meningkat dan throughput bisa sedikit menurun walaupun algoritma yang digunakan sama.

## Kesimpulan

Dengan melihat keseluruhan grafik, dapat disimpulkan bahwa hasil benchmark sudah konsisten dan tidak menunjukkan anomali besar. Kenaikan latensi terhadap ukuran data, sedikit penurunan throughput pada data besar, serta fluktuasi kecil pada data berukuran sangat kecil merupakan hal yang normal pada implementasi block cipher edukatif seperti ini.

Jadi, bentuk grafik benchmark yang dihasilkan dapat dianggap valid dan sesuai dengan karakteristik implementasi program. Hasil ini juga mendukung bahwa sistem enkripsi-dekripsi yang dibuat telah berjalan stabil pada semua mode operasi yang diuji, yaitu `CBC`, `CFB`, `OFB`, dan `CTR`.
