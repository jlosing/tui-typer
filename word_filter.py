with open("./english_5k.txt", "r", encoding="utf-8") as infile:
    with open("english_short.txt", "w", encoding="utf-8") as outfile:
        for line in infile:
            if line.__len__() < 7:
                outfile.write(line.lower())
