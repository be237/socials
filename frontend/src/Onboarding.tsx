import { useState, useEffect, useRef, useCallback } from 'react'

const API = 'http://localhost:3000/api'

type Step = 'choose' | 'telegram-phone' | 'telegram-code' | 'whatsapp-qr' | 'whatsapp-code'

interface OnboardingProps {
  onComplete: () => void
}

export default function Onboarding({ onComplete }: OnboardingProps) {
  const [step, setStep] = useState<Step>('choose')
  const [phone, setPhone] = useState('')
  const [code, setCode] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  // WhatsApp state
  const [qrData, setQrData] = useState<string | null>(null)
  const [waStatus, setWaStatus] = useState('')
  const [waPhone, setWaPhone] = useState('')
  const [pairingCode, setPairingCode] = useState<string | null>(null)
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const stopPolling = useCallback(() => {
    if (pollRef.current) {
      clearInterval(pollRef.current)
      pollRef.current = null
    }
  }, [])

  useEffect(() => {
    return () => stopPolling()
  }, [stopPolling])

  const handleTelegramRequestCode = async () => {
    if (!phone.trim()) return
    setLoading(true)
    setError('')
    try {
      const r = await fetch(`${API}/auth/telegram/request-code`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ phone: phone.trim() }),
      })
      const d = await r.json()
      if (d.success) {
        setStep('telegram-code')
      } else {
        setError(d.error || "Erreur lors de l'envoi du code")
      }
    } catch {
      setError('Erreur de connexion au serveur')
    } finally {
      setLoading(false)
    }
  }

  const handleTelegramSignIn = async () => {
    if (!code.trim()) return
    setLoading(true)
    setError('')
    try {
      const r = await fetch(`${API}/auth/telegram/sign-in`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code: code.trim() }),
      })
      const d = await r.json()
      if (d.success) {
        onComplete()
      } else {
        setError(d.error || 'Code incorrect')
      }
    } catch {
      setError('Erreur de connexion au serveur')
    } finally {
      setLoading(false)
    }
  }

  const handleWhatsAppStart = async () => {
    setLoading(true)
    setError('')
    setQrData(null)
    setPairingCode(null)
    setWaStatus('Démarrage...')
    try {
      const r = await fetch(`${API}/auth/whatsapp/start`, { method: 'POST' })
      const d = await r.json()
      if (!d.success) {
        setError(d.error || 'Erreur WhatsApp')
        setLoading(false)
        return
      }
      // Start polling for QR
      pollRef.current = setInterval(pollWhatsAppQR, 2000)
    } catch {
      setError('Impossible de contacter le serveur WhatsApp')
      setLoading(false)
    }
  }

  const pollWhatsAppQR = async () => {
    try {
      const r = await fetch(`${API}/auth/whatsapp/qr`)
      const d = await r.json()
      if (d.success && d.data) {
        setWaStatus(d.data.status)
        if (d.data.qr) {
          setQrData(d.data.qr)
          setLoading(false)
        }
      }
    } catch {
      // Sidecar might not be running
    }
  }

  const handleWhatsAppPairCode = async () => {
    if (!waPhone.trim()) return
    setLoading(true)
    setError('')
    try {
      const r = await fetch(`${API}/auth/whatsapp/pair-code`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ phone_number: waPhone.trim() }),
      })
      const d = await r.json()
      if (d.success && d.data) {
        setPairingCode(d.data)
        stopPolling()
        setLoading(false)
      } else {
        setError(d.error || 'Erreur code de liaison')
        setLoading(false)
      }
    } catch {
      setError('Impossible de contacter le serveur')
      setLoading(false)
    }
  }

  // Poll WhatsApp status when on QR/code steps
  useEffect(() => {
    if (step === 'whatsapp-qr' && !pollRef.current) {
      handleWhatsAppStart()
    }
    if (step !== 'whatsapp-qr' && step !== 'whatsapp-code') {
      stopPolling()
    }
  }, [step])

  if (step === 'choose') {
    return (
      <div className="flex h-screen bg-gray-950 text-white font-sans items-center justify-center">
        <div className="w-full max-w-md p-8">
          <div className="text-center mb-8">
            <div className="text-5xl mb-4">💬</div>
            <h1 className="text-2xl font-bold mb-2">Bienvenue sur Socials</h1>
            <p className="text-gray-400">Connectez vos messageries en quelques clics</p>
          </div>

          <div className="space-y-3">
            <button
              onClick={() => setStep('telegram-phone')}
              className="w-full flex items-center gap-4 p-4 bg-gray-800 hover:bg-gray-750 border border-gray-700 hover:border-blue-500 rounded-xl transition-all group"
            >
              <div className="w-12 h-12 rounded-full bg-blue-500 flex items-center justify-center text-xl">
                ✈️
              </div>
              <div className="text-left">
                <div className="font-semibold group-hover:text-blue-400 transition-colors">Connecter Telegram</div>
                <div className="text-xs text-gray-500">Comme Telegram Web — numéro + code</div>
              </div>
            </button>

            <button
              onClick={() => setStep('whatsapp-qr')}
              className="w-full flex items-center gap-4 p-4 bg-gray-800 hover:bg-gray-750 border border-gray-700 hover:border-green-500 rounded-xl transition-all group"
            >
              <div className="w-12 h-12 rounded-full bg-green-500 flex items-center justify-center text-xl">
                📱
              </div>
              <div className="text-left">
                <div className="font-semibold group-hover:text-green-400 transition-colors">Connecter WhatsApp</div>
                <div className="text-xs text-gray-500">Comme WhatsApp Web — QR code</div>
              </div>
            </button>

            <div className="relative my-6">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-gray-800"></div>
              </div>
              <div className="relative flex justify-center text-xs">
                <span className="bg-gray-950 px-3 text-gray-600">ou</span>
              </div>
            </div>

            <button
              onClick={onComplete}
              className="w-full flex items-center gap-4 p-4 bg-gray-900 hover:bg-gray-800 border border-gray-800 hover:border-gray-600 rounded-xl transition-all group"
            >
              <div className="w-12 h-12 rounded-full bg-gray-700 flex items-center justify-center text-xl">
                ⚙️
              </div>
              <div className="text-left">
                <div className="font-semibold group-hover:text-gray-300 transition-colors">Utiliser un Bot</div>
                <div className="text-xs text-gray-500">Mode avancé — nécessite un token BotFather</div>
              </div>
            </button>
          </div>
        </div>
      </div>
    )
  }

  if (step === 'telegram-phone') {
    return (
      <div className="flex h-screen bg-gray-950 text-white font-sans items-center justify-center">
        <div className="w-full max-w-md p-8">
          <button
            onClick={() => { setStep('choose'); setError('') }}
            className="text-gray-500 hover:text-gray-300 text-sm mb-6 flex items-center gap-1"
          >
            ← Retour
          </button>

          <div className="text-center mb-8">
            <div className="w-16 h-16 rounded-full bg-blue-500 flex items-center justify-center text-3xl mx-auto mb-4">
              ✈️
            </div>
            <h1 className="text-xl font-bold mb-2">Connexion Telegram</h1>
            <p className="text-gray-400 text-sm">
              Entrez votre numéro de téléphone. Vous recevrez un code de vérification dans l'app Telegram.
            </p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Numéro de téléphone</label>
              <input
                type="tel"
                value={phone}
                onChange={e => setPhone(e.target.value)}
                placeholder="+33 6 12 34 56 78"
                className="w-full bg-gray-800 rounded-xl px-4 py-3 text-center text-lg tracking-wider placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
                autoFocus
              />
            </div>

            {error && (
              <div className="bg-red-950/50 border border-red-900 rounded-lg p-3 text-sm text-red-300">
                {error}
              </div>
            )}

            <button
              onClick={handleTelegramRequestCode}
              disabled={!phone.trim() || loading}
              className="w-full bg-blue-600 hover:bg-blue-500 disabled:bg-gray-700 disabled:cursor-not-allowed py-3 rounded-xl font-medium transition-colors"
            >
              {loading ? 'Envoi en cours...' : 'Envoyer le code'}
            </button>
          </div>
        </div>
      </div>
    )
  }

  if (step === 'telegram-code') {
    return (
      <div className="flex h-screen bg-gray-950 text-white font-sans items-center justify-center">
        <div className="w-full max-w-md p-8">
          <button
            onClick={() => { setStep('telegram-phone'); setError('') }}
            className="text-gray-500 hover:text-gray-300 text-sm mb-6 flex items-center gap-1"
          >
            ← Retour
          </button>

          <div className="text-center mb-8">
            <div className="w-16 h-16 rounded-full bg-blue-500 flex items-center justify-center text-3xl mx-auto mb-4">
              ✈️
            </div>
            <h1 className="text-xl font-bold mb-2">Entrez le code</h1>
            <p className="text-gray-400 text-sm">
              Ouvrez Telegram — vous avez reçu un code de vérification. Entrez-le ci-dessous.
            </p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Code de vérification</label>
              <input
                type="text"
                value={code}
                onChange={e => setCode(e.target.value)}
                placeholder="12345"
                className="w-full bg-gray-800 rounded-xl px-4 py-3 text-center text-2xl tracking-[0.5em] font-mono placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
                autoFocus
                maxLength={5}
              />
            </div>

            {error && (
              <div className="bg-red-950/50 border border-red-900 rounded-lg p-3 text-sm text-red-300">
                {error}
              </div>
            )}

            <button
              onClick={handleTelegramSignIn}
              disabled={!code.trim() || loading}
              className="w-full bg-blue-600 hover:bg-blue-500 disabled:bg-gray-700 disabled:cursor-not-allowed py-3 rounded-xl font-medium transition-colors"
            >
              {loading ? 'Connexion...' : 'Se connecter'}
            </button>
          </div>
        </div>
      </div>
    )
  }

  if (step === 'whatsapp-qr') {
    return (
      <div className="flex h-screen bg-gray-950 text-white font-sans items-center justify-center">
        <div className="w-full max-w-md p-8">
          <button
            onClick={() => { setStep('choose'); setError(''); stopPolling() }}
            className="text-gray-500 hover:text-gray-300 text-sm mb-6 flex items-center gap-1"
          >
            ← Retour
          </button>

          <div className="text-center mb-8">
            <div className="w-16 h-16 rounded-full bg-green-500 flex items-center justify-center text-3xl mx-auto mb-4">
              📱
            </div>
            <h1 className="text-xl font-bold mb-2">Connexion WhatsApp</h1>
            <p className="text-gray-400 text-sm">
              Scannez le QR code avec votre téléphone via WhatsApp → Appareils liés → Lier un appareil.
            </p>
          </div>

          {error && (
            <div className="bg-red-950/50 border border-red-900 rounded-lg p-3 text-sm text-red-300 mb-4">
              {error}
            </div>
          )}

          <div className="bg-gray-800 rounded-xl p-8 flex items-center justify-center mb-6">
            {qrData ? (
              <img
                src={`https://api.qrserver.com/v1/create-qr-code/?size=250x250&data=${encodeURIComponent(qrData)}`}
                alt="QR Code WhatsApp"
                className="rounded-lg"
              />
            ) : (
              <div className="text-gray-500 text-sm text-center">
                <div className="text-4xl mb-3 animate-pulse">📷</div>
                {loading ? 'Génération du QR code...' : 'En attente du QR code...'}
                <br />
                <span className="text-xs text-gray-600">{waStatus}</span>
              </div>
            )}
          </div>

          <div className="text-center">
            <button
              onClick={() => { setStep('whatsapp-code'); stopPolling() }}
              className="text-blue-400 hover:text-blue-300 text-sm underline"
            >
              Utiliser un code de liaison à la place
            </button>
          </div>
        </div>
      </div>
    )
  }

  if (step === 'whatsapp-code') {
    return (
      <div className="flex h-screen bg-gray-950 text-white font-sans items-center justify-center">
        <div className="w-full max-w-md p-8">
          <button
            onClick={() => { setStep('whatsapp-qr'); setError(''); setPairingCode(null) }}
            className="text-gray-500 hover:text-gray-300 text-sm mb-6 flex items-center gap-1"
          >
            ← Retour
          </button>

          <div className="text-center mb-8">
            <div className="w-16 h-16 rounded-full bg-green-500 flex items-center justify-center text-3xl mx-auto mb-4">
              📱
            </div>
            <h1 className="text-xl font-bold mb-2">Code de liaison</h1>
            <p className="text-gray-400 text-sm">
              Allez dans WhatsApp → Appareils liés → Lier un appareil → Lier avec un numéro de téléphone.
            </p>
          </div>

          {error && (
            <div className="bg-red-950/50 border border-red-900 rounded-lg p-3 text-sm text-red-300 mb-4">
              {error}
            </div>
          )}

          {pairingCode ? (
            <div className="bg-gray-800 rounded-xl p-8 flex flex-col items-center justify-center mb-6">
              <div className="text-xs text-gray-400 mb-2">Votre code de liaison</div>
              <div className="text-3xl font-mono font-bold tracking-[0.3em] text-green-400">
                {pairingCode}
              </div>
              <div className="text-xs text-gray-500 mt-3">Entrez ce code sur votre téléphone</div>
            </div>
          ) : (
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-2">Numéro de téléphone</label>
                <input
                  type="tel"
                  value={waPhone}
                  onChange={e => setWaPhone(e.target.value)}
                  placeholder="33612345678"
                  className="w-full bg-gray-800 rounded-xl px-4 py-3 text-center text-lg tracking-wider placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-green-500/50 transition-all"
                  autoFocus
                />
              </div>
              <button
                onClick={handleWhatsAppPairCode}
                disabled={!waPhone.trim() || loading}
                className="w-full bg-green-600 hover:bg-green-500 disabled:bg-gray-700 disabled:cursor-not-allowed py-3 rounded-xl font-medium transition-colors"
              >
                {loading ? 'Génération...' : 'Obtenir le code'}
              </button>
            </div>
          )}
        </div>
      </div>
    )
  }

  return null
}
