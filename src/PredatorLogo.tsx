export default function PredatorLogo({ className = "w-10 h-10" }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 256 256"
      className={className}
      fill="none"
    >
      <defs>
        <linearGradient id="psShieldBg" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor="#222832" />
          <stop offset="35%" stopColor="#151921" />
          <stop offset="100%" stopColor="#0a0c10" />
        </linearGradient>

        <linearGradient id="psWingGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#353f4e" />
          <stop offset="60%" stopColor="#1b2029" />
          <stop offset="100%" stopColor="#0e1115" />
        </linearGradient>

        <linearGradient id="psCyanNeon" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor="#00f5ff" />
          <stop offset="50%" stopColor="#00c8e6" />
          <stop offset="100%" stopColor="#007799" />
        </linearGradient>

        <linearGradient id="psMetalLight" x1="0%" y1="0%" x2="100%" y2="50%">
          <stop offset="0%" stopColor="#ffffff" />
          <stop offset="40%" stopColor="#d8e0e8" />
          <stop offset="100%" stopColor="#909baa" />
        </linearGradient>

        <linearGradient id="psMetalDark" x1="0%" y1="0%" x2="100%" y2="50%">
          <stop offset="0%" stopColor="#707b8c" />
          <stop offset="60%" stopColor="#4a5360" />
          <stop offset="100%" stopColor="#2c333d" />
        </linearGradient>

        <filter id="psCyanGlow" x="-20%" y="-20%" width="140%" height="140%">
          <feGaussianBlur stdDeviation="4" result="blur" />
          <feComposite in="SourceGraphic" in2="blur" operator="over" />
        </filter>
        
        <filter id="psEmblemShadow" x="-20%" y="-20%" width="140%" height="140%">
          <feDropShadow dx="0" dy="5" stdDeviation="5" floodColor="#000000" floodOpacity="0.8" />
        </filter>
      </defs>

      {/* Central Shield Body */}
      <path
        d="M 52,28 L 204,28 L 194,204 L 164,228 L 92,228 L 62,204 Z" 
        fill="url(#psShieldBg)"
        stroke="#27303e"
        strokeWidth="2"
      />

      {/* Inner Armor Inset */}
      <path
        d="M 58,34 L 198,34 L 189,198 L 160,220 L 96,220 L 67,198 Z" 
        fill="#0f1218"
        opacity="0.9"
      />

      {/* Left Wing Flank with Neon Cyan Edge */}
      <path
        d="M 28,34 L 56,28 L 64,196 L 46,182 L 32,140 Z" 
        fill="url(#psWingGrad)"
        stroke="#181e28"
        strokeWidth="1.5"
      />
      <path
        d="M 38,40 Q 32,100 48,176" 
        fill="none"
        stroke="url(#psCyanNeon)"
        strokeWidth="4"
        strokeLinecap="round"
        filter="url(#psCyanGlow)"
      />
      <path
        d="M 38,40 Q 32,100 48,176" 
        fill="none"
        stroke="#e0ffff"
        strokeWidth="1.2"
        strokeLinecap="round"
      />

      {/* Right Wing Flank with Neon Cyan Edge */}
      <path
        d="M 228,34 L 200,28 L 192,196 L 210,182 L 224,140 Z" 
        fill="url(#psWingGrad)"
        stroke="#181e28"
        strokeWidth="1.5"
      />
      <path
        d="M 218,40 Q 224,100 208,176" 
        fill="none"
        stroke="url(#psCyanNeon)"
        strokeWidth="4"
        strokeLinecap="round"
        filter="url(#psCyanGlow)"
      />
      <path
        d="M 218,40 Q 224,100 208,176" 
        fill="none"
        stroke="#e0ffff"
        strokeWidth="1.2"
        strokeLinecap="round"
      />

      {/* Iconic Predator 3D Metallic Crest */}
      <g filter="url(#psEmblemShadow)">
        {/* Left Facets (Light Silver) */}
        <polygon points="128,78 128,52 82,42 74,124 94,142 96,112 110,132" fill="url(#psMetalLight)" />
        <polygon points="128,78 110,132 114,178 128,156" fill="url(#psMetalLight)" />
        <polygon points="74,124 90,126 84,102" fill="url(#psMetalLight)" opacity="0.9" />
        <polygon points="94,142 98,174 114,178 108,142" fill="url(#psMetalLight)" />

        {/* Right Facets (Dark Steel Shadow) */}
        <polygon points="128,78 128,52 174,42 182,124 162,142 160,112 146,132" fill="url(#psMetalDark)" />
        <polygon points="128,78 146,132 142,178 128,156" fill="url(#psMetalDark)" />
        <polygon points="182,124 166,126 172,102" fill="url(#psMetalDark)" opacity="0.9" />
        <polygon points="162,142 158,174 142,178 148,142" fill="url(#psMetalDark)" />

        {/* Center Ridge Line */}
        <line x1="128" y1="52" x2="128" y2="156" stroke="#ffffff" strokeWidth="1.2" opacity="0.85" />
      </g>
    </svg>
  );
}
