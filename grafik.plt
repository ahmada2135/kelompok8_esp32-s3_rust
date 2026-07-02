# Konfigurasi output menjadi gambar PNG
set terminal pngcairo size 1024,768 enhanced font 'Arial,12'
set output 'grafik_sensor.png'

# Judul dan Label Sumbu
set title "Grafik Pembacaan Sensor Suhu, Kelembapan, dan Gas terhadap Siklus" font "Arial,14"
set xlabel "Siklus"

# Sumbu Y1 (Suhu & Kelembapan)
set ylabel "Suhu (°C) & Kelembapan (%)"
set yrange [0:100]
set ytics 10 nomirror
set grid ytics

# Sumbu Y2 (Gas)
set y2label "Konsentrasi Gas (PPM)" textcolor rgb "forest-green"
set y2range [0:1000]
set y2tics 100
set grid y2tics

# Posisi Legend
set key outside right top

# Perintah Plotting (Kolom 1:Siklus, 2:Suhu, 3:Hum, 4:Gas)
plot 'data_sensor.txt' using 1:2 axes x1y1 with linespoints linewidth 2 linecolor rgb "red" title "Suhu (°C)", \
     'data_sensor.txt' using 1:3 axes x1y1 with linespoints linewidth 2 linecolor rgb "blue" title "Kelembapan (%)", \
     'data_sensor.txt' using 1:4 axes x1y2 with linespoints linewidth 2 linecolor rgb "forest-green" title "Gas (PPM)"