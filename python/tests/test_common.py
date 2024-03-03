from rs1090 import decode


def test_crc() -> None:
    assert decode("8D406B902015A678D4D220AA4BDA")["DF"] == "ADSB"
    assert decode("8d8960ed58bf053cf11bc5932b7d")["DF"] == "ADSB"
    assert decode("8d45cab390c39509496ca9a32912")["DF"] == "ADSB"
    assert decode("8d49d3d4e1089d00000000744c3b")["DF"] == "ADSB"
    assert decode("8d74802958c904e6ef4ba0184d5c")["DF"] == "ADSB"
    assert decode("8d4400cd9b0000b4f87000e71a10")["DF"] == "ADSB"
    assert decode("8d4065de58a1054a7ef0218e226a")["DF"] == "ADSB"


def test_fail_crc() -> None:
    assert decode("8d4ca251204994b1c36e60a5343d") is None


def test_icao24():
    assert decode("8D406B902015A678D4D220AA4BDA")["icao24"] == "406b90"
    assert decode("A0001839CA3800315800007448D9")["icao24"] == "400940"
    assert decode("A000139381951536E024D4CCF6B5")["icao24"] == "3c4dd2"
    assert decode("A000029CFFBAA11E2004727281F1")["icao24"] == "4243d0"


def test_altcode():
    assert decode("A02014B400000000000000F9D514")["altitude"] == 32300


def test_idcode():
    assert decode("A800292DFFBBA9383FFCEB903D01")["squawk"] == "1346"
