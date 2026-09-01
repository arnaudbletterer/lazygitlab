import os
import fitz
from PIL import Image

def generate_branding():
    os.makedirs("docs/assets", exist_ok=True)

    # 512x512 Canvas SVG
    svg_content = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <!-- Background Slate/Obsidian Gradient -->
    <linearGradient id="bgGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#1B1E28"/>
      <stop offset="45%" stop-color="#12141C"/>
      <stop offset="100%" stop-color="#0A0B10"/>
    </linearGradient>

    <!-- Outer Squircle Border Gradient -->
    <linearGradient id="borderGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FC6D26" stop-opacity="0.9"/>
      <stop offset="50%" stop-color="#FCA326" stop-opacity="0.5"/>
      <stop offset="100%" stop-color="#E24329" stop-opacity="0.25"/>
    </linearGradient>

    <!-- Inner Terminal Border Gradient -->
    <linearGradient id="termBorderGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FC6D26" stop-opacity="0.75"/>
      <stop offset="100%" stop-color="#E24329" stop-opacity="0.35"/>
    </linearGradient>
  </defs>

  <!-- Squircle App Badge Container -->
  <rect x="18" y="18" width="476" height="476" rx="106" fill="url(#bgGrad)" stroke="url(#borderGrad)" stroke-width="3"/>

  <!-- Top Terminal Window Controls (Traffic Lights) -->
  <g opacity="0.95">
    <circle cx="68" cy="62" r="6" fill="#FF5F56"/>
    <circle cx="90" cy="62" r="6" fill="#FEBC2E"/>
    <circle cx="112" cy="62" r="6" fill="#28C840"/>
  </g>

  <!-- GitLab Fox Group -->
  <g transform="translate(46, 76) scale(17.5)">
    <!-- 1. Left Lower Cheek & Wings -->
    <path fill="#e24329" d="M12 21.603l-11.072-8.045 2.53-7.783 2.53 7.783h6.862l2.53-7.783 2.53 7.783z"/>

    <!-- 2. Right Lower Cheek & Wings -->
    <path fill="#fc6d26" d="M12 21.603l11.072-8.045-2.53-7.783-2.53 7.783h-6.862l-2.53-7.783-2.53 7.783z"/>

    <!-- 3. Center Forehead Triangle -->
    <path fill="#fca326" d="M12 21.603l-3.428-11.468h6.856z"/>

    <!-- 4. Right Ear -->
    <path fill="#fcc426" d="M23.072 13.558l-1.26-3.867-2.526-7.775a1.004 1.004 0 0 0-1.906 0l-2.526 7.775z"/>

    <!-- 5. Left Ear -->
    <path fill="#e24329" d="M.928 13.558l1.26-3.867 2.526-7.775a1.004 1.004 0 0 1 1.906 0l2.526 7.775z"/>

    <!-- Inset Terminal Console Window in Fox Face -->
    <rect x="8.5" y="11.8" width="7.0" height="5.2" rx="0.8" fill="#0C0E14" stroke="url(#termBorderGrad)" stroke-width="0.18"/>

    <!-- Terminal Subtle Header Bar in Inset Window -->
    <rect x="8.5" y="11.8" width="7.0" height="1.1" rx="0.8" fill="#161922"/>
    <circle cx="9.2" cy="12.35" r="0.22" fill="#FF5F56" opacity="0.85"/>
    <circle cx="9.8" cy="12.35" r="0.22" fill="#FEBC2E" opacity="0.85"/>
    <circle cx="10.4" cy="12.35" r="0.22" fill="#28C840" opacity="0.85"/>

    <!-- Iconic Terminal Prompt '>_' in Window -->
    <!-- Prompt Chevron '>' in Pure White -->
    <path d="M 9.7 13.9 L 10.9 14.9 L 9.7 15.9" fill="none" stroke="#FFFFFF" stroke-width="0.52" stroke-linecap="round" stroke-linejoin="round"/>

    <!-- Terminal Cursor Underscore '_' in Radiant GitLab Orange -->
    <line x1="11.8" y1="15.9" x2="14.3" y2="15.9" stroke="#FC6D26" stroke-width="0.52" stroke-linecap="round"/>
  </g>
</svg>'''

    svg_path = "docs/assets/logo.svg"
    with open(svg_path, "w", encoding="utf-8") as f:
        f.write(svg_content)
    print(f"Saved {svg_path}")

    # Render PNG at 1024x1024 with PyMuPDF (DPI=144 for 512 pt)
    doc = fitz.open(svg_path)
    pix = doc[0].get_pixmap(dpi=144)
    temp_hi = "docs/assets/logo_hi.png"
    pix.save(temp_hi)

    img = Image.open(temp_hi)

    # logo.png (512x512)
    logo_512 = img.resize((512, 512), Image.Resampling.LANCZOS)
    logo_png_path = "docs/assets/logo.png"
    logo_512.save(logo_png_path, "PNG", optimize=True)
    print(f"Saved {logo_png_path}")

    # favicon.png (128x128)
    fav_png_path = "docs/assets/favicon.png"
    fav_128 = img.resize((128, 128), Image.Resampling.LANCZOS)
    fav_128.save(fav_png_path, "PNG", optimize=True)
    print(f"Saved {fav_png_path}")

    # favicon.ico (multi-resolution 16, 32, 48, 64)
    fav_ico_path = "docs/assets/favicon.ico"
    ico_sizes = [(16, 16), (32, 32), (48, 48), (64, 64)]
    img.save(fav_ico_path, format="ICO", sizes=ico_sizes)
    print(f"Saved {fav_ico_path}")

    if os.path.exists(temp_hi):
        os.remove(temp_hi)

if __name__ == "__main__":
    generate_branding()
