import sys
import os

from Crypto.Cipher import AES
from Crypto.Random import get_random_bytes

key = get_random_bytes(16)
iv = get_random_bytes(16)
original = 'original.rgba'

def png_to_rgba():
    os.system('convert -depth 32 original.png original.rgba')


def enc_ecb():
    output_file = 'ecb.rgba'

    cipher = AES.new(key, AES.MODE_ECB)
    with open(original, 'rb') as f:
        content = f.read()
        ciphertext = cipher.encrypt(content)
        with open(output_file, 'wb') as wf:
            wf.write(ciphertext)


def enc_cbc():
    output_file = 'cbc.rgba'

    cipher = AES.new(key, AES.MODE_CBC, IV=iv)
    with open(original, 'rb') as f:
        content = f.read()
        ciphertext = cipher.encrypt(content)
        with open(output_file, 'wb') as wf:
            wf.write(ciphertext)


def rgba_to_png(filename):
    os.system('convert -size 512x512 -depth 32 {}.rgba {}.png'.format(
        filename, filename))


def main():
    png_to_rgba()
    enc_ecb()
    enc_cbc()
    rgba_to_png('ecb')
    rgba_to_png('cbc')


if __name__ == '__main__':
    main()
