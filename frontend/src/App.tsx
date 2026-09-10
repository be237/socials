import { useState, useEffect, useRef } from 'react'

interface Message {
  id: string
  conversation_id: string
  sender_id: string
  content: string
  message_type: string
  status: string
  created_at: string
  connector_id: string
}

interface Conversation {
  id: string
  title: string
  avatar_url?: string | null
  conversation_type: string
  last_message_at: string | null
}

interface Channel {
  id: string
  name: string
  connected: boolean
  syncing?: boolean
  sync_error?: string | null
  last_sync_at?: string | null
}

const API = 'http://localhost:3000/api'

function isTelegram(title: string): boolean {
  return title.startsWith('TG:') || title.toLowerCase().includes('telegram')
}

function formatConversationTitle(title: string): string {
  // Transform "TG:8925002788:@username" → "@username"
  if (title.startsWith('TG:')) {
    const parts = title.split(':')
    // parts[0] = "TG", parts[1] = chat_id, parts[2..] = display name (may contain colons)
    if (parts.length >= 3) {
      return parts.slice(2).join(':')
    }

  }
  return title
}

function getConnectorIcon(title: string): string {
  if (isTelegram(title)) return '✈️'
  if (title.toLowerCase().includes('discord')) return '🎮'
  if (title.toLowerCase().includes('whatsapp')) return '📱'
  if (title.toLowerCase().includes('gmail') || title.toLowerCase().includes('email')) return '📧'
  return '💬'
}

function getConnectorColor(title: string): string {
  if (isTelegram(title)) return 'bg-blue-500'
  if (title.toLowerCase().includes('discord')) return 'bg-indigo-500'
  if (title.toLowerCase().includes('whatsapp')) return 'bg-green-500'
  if (title.toLowerCase().includes('gmail') || title.toLowerCase().includes('email')) return 'bg-red-500'
  return 'bg-gray-500'
}

function ConversationAvatar({ conversation }: { conversation: Conversation }) {
  return conversation.avatar_url ? (
    <img src={conversation.avatar_url} alt="" className="h-10 w-10 shrink-0 rounded-xl object-cover" />
  ) : (
    <div className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ${getConnectorColor(conversation.title)} text-base`}>
      {getConnectorIcon(conversation.title)}
    </div>
  )
}

export default function App() {
  const [hasStarted, setHasStarted] = useState(() => localStorage.getItem('socials-started') === 'true')
  const [welcomeName, setWelcomeName] = useState('')
  const [welcomeEmail, setWelcomeEmail] = useState('')
  const [welcomeError, setWelcomeError] = useState('')
  const [conversations, setConversations] = useState<Conversation[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
  const [newMsg, setNewMsg] = useState('')
  const [search, setSearch] = useState('')
  const [showNewChat, setShowNewChat] = useState(false)
  const [activeSection, setActiveSection] = useState<'conversations' | 'channels' | 'settings'>('conversations')
  const [notificationsEnabled, setNotificationsEnabled] = useState(
    () => localStorage.getItem('socials-notifications') !== 'false'
  )
  const [compactMode, setCompactMode] = useState(
    () => localStorage.getItem('socials-compact') === 'true'
  )
  const [lightTheme, setLightTheme] = useState(
    () => localStorage.getItem('socials-theme') !== 'dark'
  )
  const [channels, setChannels] = useState<Channel[]>([])
  const [telegramPhone, setTelegramPhone] = useState('')
  const [telegramCode, setTelegramCode] = useState('')
  const [telegramPassword, setTelegramPassword] = useState('')
  const [telegramStep, setTelegramStep] = useState<'phone' | 'code' | 'password'>('phone')
  const [channelError, setChannelError] = useState('')
  const [isConnectingChannel, setIsConnectingChannel] = useState(false)
  const [newChatTitle, setNewChatTitle] = useState('')
  const [isOnline, setIsOnline] = useState<boolean | null>(null)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const loadConversations = async () => {
    const r = await fetch(`${API}/conversations`)
    if (!r.ok) throw new Error('API non joignable')
    const d = await r.json()
    setConversations(d.data || [])
    setIsOnline(true)
  }

  const loadMessages = async (conversationId: string) => {
    const r = await fetch(`${API}/conversations/${conversationId}/messages`)
    if (!r.ok) throw new Error('Messages non disponibles')
    const d = await r.json()
    setMessages(d.data || [])
  }

  const selectConversation = (conversationId: string) => {
    setMessages([])
    setSelected(conversationId)
  }

  useEffect(() => {
    Promise.resolve().then(() => loadConversations()).catch(() => console.error('Impossible de charger les conversations'))
    const loadChannels = () => fetch(`${API}/channels`)
      .then(response => response.json())
      .then(payload => setChannels(payload.data || []))
    loadChannels().catch(() => console.error('Impossible de charger les canaux'))
    const channelTimer = window.setInterval(() => {
      loadChannels().catch(() => undefined)
    }, 2500)
    return () => window.clearInterval(channelTimer)
  }, [])

  useEffect(() => {
    if (!selected) {
      return
    }

    Promise.resolve()
      .then(() => loadMessages(selected))
      .catch(err => console.error("Erreur lors de la récupération des messages:", err))
  }, [selected])

  useEffect(() => {
    let socket: WebSocket | null = null
    let reconnectTimer: number | undefined
    let stopped = false

    const connect = () => {
      if (stopped) return
      const wsUrl = API.replace(/^http/, 'ws').replace(/\/api$/, '/api/ws')
      socket = new WebSocket(wsUrl)
      socket.onopen = () => setIsOnline(true)
      socket.onmessage = event => {
        try {
          const envelope = JSON.parse(event.data)
          const eventName = typeof envelope.event === 'string'
            ? envelope.event
            : Object.keys(envelope.event || {})[0]
          if (eventName === 'MessageReceived' || eventName === 'MessageSent' ||
              eventName === 'ConversationCreated' || eventName === 'ConversationUpdated') {
            loadConversations().catch(() => setIsOnline(false))
          }
          const conversationId = envelope.conversation_id ||
            envelope.event?.conversation_id ||
            envelope.message?.conversation_id
          if (selected && conversationId === selected &&
              (eventName === 'MessageReceived' || eventName === 'MessageSent')) {
            loadMessages(selected).catch(err => console.error("Erreur de synchronisation:", err))
          }
        } catch (err) {
          console.error("Événement serveur invalide:", err)
        }
      }
      socket.onclose = () => {
        if (!stopped) {
          setIsOnline(false)
          reconnectTimer = window.setTimeout(connect, 2000)
        }
      }
      socket.onerror = () => socket?.close()
    }

    connect()
    return () => {
      stopped = true
      if (reconnectTimer !== undefined) window.clearTimeout(reconnectTimer)
      socket?.close()
    }
  }, [selected])

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const handleSend = async () => {
    if (!newMsg.trim() || !selected) return
    const textToSend = newMsg
    setNewMsg('')

    try {
      const response = await fetch(`${API}/messages`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ conversation_id: selected, content: textToSend, sender_name: 'Vous' }),
      })
      if (!response.ok) throw new Error("Envoi refusé")
      await loadMessages(selected)
    } catch (err) {
      console.error("Erreur d'envoi:", err)
    }
  }

  const handleCreateChat = async () => {
    if (!newChatTitle.trim()) return
    try {
      const r = await fetch(`${API}/conversations`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title: newChatTitle, conversation_type: 'private' }),
      })
      const d = await r.json()
      if (d.data) {
        setConversations(prev => [d.data, ...prev])
        selectConversation(d.data.id)
        setNewChatTitle('')
        setShowNewChat(false)
      }

    } catch (err) {
      console.error("Erreur création conversation:", err)
    }
  }

  const handleConnectTelegram = async () => {
    if (!telegramPhone.trim() && telegramStep === 'phone') return
    setIsConnectingChannel(true)
    setChannelError('')
    try {
      const endpoint = telegramStep === 'phone' ? 'phone' : 'code'
      const body = telegramStep === 'phone'
        ? { phone: telegramPhone.trim() }
        : { code: telegramCode.trim(), password: telegramPassword || undefined }
      const response = await fetch(`${API}/channels/telegram/${endpoint}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })
      const payload = await response.json()
      if (!response.ok) throw new Error(payload.detail || 'Connexion Telegram refusée')
      if (payload.data.step === 'code') setTelegramStep('code')
      if (payload.data.step === 'password') setTelegramStep('password')
      if (payload.data.step === 'connected') {
        setChannels(prev => prev.map(channel =>
          channel.id === 'telegram' ? { ...channel, connected: true } : channel
        ))
        setTelegramPhone('')
        setTelegramCode('')
        setTelegramPassword('')
        setTelegramStep('phone')
      }

    } catch (error) {
      setChannelError(error instanceof Error ? error.message : 'Connexion impossible')
    } finally {
      setIsConnectingChannel(false)
    }
  }

  const handleDisconnectTelegram = async () => {
    setChannelError('')
    try {
      const response = await fetch(`${API}/channels/telegram/disconnect`, { method: 'POST' })
      const payload = await response.json()
      if (!response.ok) throw new Error(payload.detail || 'Déconnexion Telegram refusée')
      setChannels(prev => prev.map(channel =>
        channel.id === 'telegram' ? { ...channel, connected: false } : channel
      ))
      setTelegramStep('phone')
    } catch (error) {
      setChannelError(error instanceof Error ? error.message : 'Déconnexion impossible')
    }
  }

  const filtered = conversations.filter(c =>
    c.title.toLowerCase().includes(search.toLowerCase())
  )

  const selectedConv = conversations.find(c => c.id === selected)

  const handleStart = () => {
    if (!welcomeEmail.trim() || !welcomeEmail.includes('@')) {
      setWelcomeError('Entre une adresse e-mail valide pour commencer.')
      return
    }

    localStorage.setItem('socials-started', 'true')
    localStorage.setItem('socials-user-email', welcomeEmail.trim())
    localStorage.setItem('socials-user-name', welcomeName.trim() || 'Utilisateur')
    setHasStarted(true)
  }

  const openSection = (section: 'conversations' | 'channels' | 'settings') => {
    setActiveSection(section)
    if (section !== 'conversations') setShowNewChat(false)
  }

  const toggleNotifications = () => {
    setNotificationsEnabled(value => {
      localStorage.setItem('socials-notifications', String(!value))
      return !value
    })
  }

  const toggleCompactMode = () => {
    setCompactMode(value => {
      localStorage.setItem('socials-compact', String(!value))
      return !value
    })
  }

  const toggleTheme = () => {
    setLightTheme(value => {
      localStorage.setItem('socials-theme', value ? 'dark' : 'light')
      return !value
    })
  }

  if (!hasStarted) {
    return (
      <div className="min-h-screen bg-gray-950 text-white flex items-center justify-center px-6">
        <div className="w-full max-w-md">
          <div className="text-center mb-8">
            <div className="text-6xl mb-4">💬</div>
            <h1 className="text-3xl font-bold tracking-tight">Bienvenue dans Socials</h1>
            <p className="text-gray-400 mt-3">
              Centralise tes conversations dans une seule application.
            </p>
          </div>
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 shadow-2xl">
            <h2 className="text-lg font-semibold mb-1">Commencer</h2>
            <p className="text-sm text-gray-500 mb-5">
              Configure ton espace local. Tes données restent sur cet ordinateur.
            </p>
            <label className="block text-sm text-gray-300 mb-2" htmlFor="welcome-name">
              Nom
            </label>
            <input
              id="welcome-name"
              type="text"
              placeholder="Ton nom"
              value={welcomeName}
              onChange={e => setWelcomeName(e.target.value)}
              className="w-full bg-gray-800 rounded-lg px-3 py-3 mb-4 text-sm placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
              autoFocus
            />
            <label className="block text-sm text-gray-300 mb-2" htmlFor="welcome-email">
              Adresse e-mail
            </label>
            <input
              id="welcome-email"
              type="email"
              placeholder="toi@exemple.com"
              value={welcomeEmail}
              onChange={e => { setWelcomeEmail(e.target.value); setWelcomeError('') }}
              onKeyDown={e => e.key === 'Enter' && handleStart()}
              className="w-full bg-gray-800 rounded-lg px-3 py-3 text-sm placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            {welcomeError && <p className="text-sm text-rose-400 mt-2">{welcomeError}</p>}
            <button
              onClick={handleStart}
              className="w-full bg-blue-600 hover:bg-blue-500 rounded-lg px-4 py-3 mt-5 font-medium transition-colors"
            >
              Ouvrir mon espace
            </button>
            <p className="text-xs text-gray-600 text-center mt-4">
              Aucun compte distant requis pour utiliser Socials en local.
            </p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={`${lightTheme ? 'theme-light' : ''} flex h-screen min-w-0 overflow-hidden bg-[#0b1220] text-slate-100 font-sans`}>
      {/* Main chat area */}
      <div className="relative min-h-0 min-w-0 flex-1 flex flex-col bg-[#0f172a] pb-[94px]">
        {activeSection === 'channels' ? (
          <div className="flex-1 overflow-y-auto">
            <div className="flex h-[72px] items-center border-b border-slate-800 bg-[#111827] px-5 sm:px-8">
              <div>
                <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-blue-400">
                  Socials
                  <span className={`h-2 w-2 rounded-full ${
                    isOnline === true ? 'bg-emerald-500' : isOnline === false ? 'bg-rose-500' : 'bg-amber-500 animate-pulse'
                  }`} title={isOnline === true ? 'Connecté' : isOnline === false ? 'Déconnecté' : 'Connexion...'} />
                </div>
                <div className="mt-0.5 text-lg font-semibold text-slate-100">Canaux connectés</div>
              </div>
            </div>
            <div className="p-4 sm:p-8 lg:p-12">
            <div className="max-w-4xl mx-auto">
              <div className="flex items-start justify-between gap-4 mb-8 sm:mb-10 border-b border-slate-800 pb-7">
                <div>
                  <div className="text-xs uppercase tracking-[0.18em] text-blue-400 font-semibold mb-2">Espace de travail</div>
                  <h2 className="text-2xl sm:text-3xl font-bold tracking-tight">Canaux connectés</h2>
                  <p className="text-sm sm:text-base text-slate-400 mt-2">Connecte tes services pour retrouver tes conversations au même endroit.</p>
                </div>
                <button onClick={() => openSection('conversations')} className="text-slate-500 hover:text-white text-2xl">×</button>
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                {channels.map(channel => (
                  <div key={channel.id} className="bg-[#111827] border border-slate-800 rounded-xl p-5 shadow-xl">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-4">
                        <div className="w-11 h-11 rounded-xl bg-blue-600 flex items-center justify-center text-xl">✈</div>
                        <div>
                          <div className="font-semibold text-lg">{channel.name}</div>
                          <div className="text-sm text-slate-500">Messages et conversations</div>
                        </div>
                      </div>
                      <span className={`text-xs px-2.5 py-1 rounded-full ${
                        channel.syncing
                          ? 'bg-amber-500/15 text-amber-400'
                          : channel.connected
                            ? 'bg-emerald-500/15 text-emerald-400'
                            : 'bg-slate-700 text-slate-400'
                      }`}>
                        {channel.syncing ? 'Synchronisation...' : channel.connected ? 'Connecté' : 'À connecter'}
                      </span>
                    </div>
                    {channel.last_sync_at && !channel.syncing && (
                      <div className="mt-3 text-xs text-slate-500">
                        Dernière synchronisation : {new Date(channel.last_sync_at).toLocaleString('fr-FR', {
                          dateStyle: 'short',
                          timeStyle: 'short',
                        })}
                      </div>
                    )}
                    {channel.sync_error && (
                      <div className="mt-3 rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-xs text-rose-400">
                        Synchronisation interrompue : {channel.sync_error}
                      </div>
                    )}
                    {channel.connected && (
                      <div className="mt-6 flex items-center justify-between border-t border-slate-800 pt-5">
                        <div className="text-sm text-slate-400">Session Telegram restaurée automatiquement.</div>
                        <button
                          onClick={handleDisconnectTelegram}
                          className="rounded-xl border border-rose-500/30 px-3 py-2 text-sm text-rose-400 transition-colors hover:bg-rose-500/10"
                        >
                          Déconnecter
                        </button>
                      </div>
                    )}
                    {!channel.connected && (
                      <div className="mt-6 pt-5 border-t border-slate-800">
                        <p className="text-sm text-slate-400 mb-4">Connexion sécurisée comme Telegram Web. Aucun bot ni token API à créer.</p>
                        {telegramStep === 'phone' && <input type="tel" placeholder="+33 6 12 34 56 78" value={telegramPhone} onChange={event => { setTelegramPhone(event.target.value); setChannelError('') }} className="w-full bg-slate-950 border border-slate-700 rounded-xl px-4 py-3 text-sm placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-blue-500/50" />}
                        {telegramStep !== 'phone' && <input type="text" inputMode="numeric" placeholder="Code reçu dans Telegram" value={telegramCode} onChange={event => { setTelegramCode(event.target.value); setChannelError('') }} className="w-full bg-slate-950 border border-slate-700 rounded-xl px-4 py-3 text-sm placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-blue-500/50" />}
                        {telegramStep === 'password' && <input type="password" placeholder="Mot de passe Telegram 2FA" value={telegramPassword} onChange={event => { setTelegramPassword(event.target.value); setChannelError('') }} className="w-full bg-slate-950 border border-slate-700 rounded-xl px-4 py-3 mt-3 text-sm placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-blue-500/50" />}
                        <button onClick={handleConnectTelegram} disabled={isConnectingChannel || (telegramStep === 'phone' ? !telegramPhone.trim() : !telegramCode.trim())} className="w-full bg-blue-600 disabled:bg-slate-700 disabled:text-slate-500 hover:bg-blue-500 rounded-xl px-4 py-3 mt-3 text-sm font-medium transition-colors">
                          {isConnectingChannel ? 'Connexion...' : telegramStep === 'phone' ? 'Envoyer le code' : 'Valider le code'}
                        </button>
                        {channelError && <p className="text-sm text-rose-400 mt-3">{channelError}</p>}
                      </div>
                    )}
                  </div>
                ))}
                <div className="border border-dashed border-slate-700 rounded-xl p-6 flex items-center justify-center min-h-[170px]">
                  <button className="text-slate-500 hover:text-blue-300 text-sm">+ Ajouter un autre service (bientôt)</button>
                </div>
              </div>
            </div>
            </div>
          </div>
        ) : activeSection === 'settings' ? (
          <div className="flex-1 overflow-y-auto">
            <div className="flex h-[72px] items-center border-b border-slate-800 bg-[#111827] px-5 sm:px-8">
              <div>
                <div className="text-xs font-medium uppercase tracking-[0.18em] text-blue-400">Socials</div>
                <div className="mt-0.5 text-lg font-semibold text-slate-100">Paramètres</div>
              </div>
            </div>
            <div className="p-4 sm:p-8 lg:p-12">
            <div className="max-w-3xl mx-auto">
              <div className="mb-10">
                <div className="text-xs uppercase tracking-[0.18em] text-blue-400 font-semibold mb-2">Préférences</div>
                <h2 className="text-2xl sm:text-3xl font-bold tracking-tight">Paramètres</h2>
                <p className="text-sm sm:text-base text-slate-400 mt-2">Gère ton profil et ton expérience Socials.</p>
              </div>
              <div className="space-y-4">
                <div className="bg-[#111827] border border-slate-800 rounded-xl p-5">
                  <div className="text-sm text-slate-500 mb-3">Profil local</div>
                  <div className="text-xl font-semibold">{localStorage.getItem('socials-user-name') || 'Utilisateur'}</div>
                  <div className="text-slate-400 mt-1">{localStorage.getItem('socials-user-email') || 'Aucun e-mail renseigné'}</div>
                </div>
                <div className="bg-[#111827] border border-slate-800 rounded-xl divide-y divide-slate-800">
                  <button onClick={toggleNotifications} className="w-full flex items-center justify-between p-5 text-left hover:bg-slate-800/40">
                    <span><span className="mr-3">🔔</span>Notifications</span>
                    <span className={notificationsEnabled ? 'text-emerald-400 text-sm' : 'text-slate-500 text-sm'}>{notificationsEnabled ? 'Activées' : 'Désactivées'}</span>
                  </button>
                  <button onClick={toggleCompactMode} className="w-full flex items-center justify-between p-5 text-left hover:bg-slate-800/40">
                    <span><span className="mr-3">📏</span>Mode compact</span>
                    <span className={compactMode ? 'text-emerald-400 text-sm' : 'text-slate-500 text-sm'}>{compactMode ? 'Activé' : 'Désactivé'}</span>
                  </button>
                  <button onClick={toggleTheme} className="w-full flex items-center justify-between p-5 text-left hover:bg-slate-800/40">
                    <span><span className="mr-3">◐</span>Thème clair</span>
                    <span className={lightTheme ? 'text-blue-500 text-sm' : 'text-slate-500 text-sm'}>{lightTheme ? 'Activé' : 'Désactivé'}</span>
                  </button>
                  <button onClick={() => { localStorage.removeItem('socials-started'); window.location.reload() }} className="w-full text-left p-5 text-rose-400 hover:bg-rose-950/20">
                    <span className="mr-3">⎋</span>Réinitialiser l’espace local
                  </button>
                </div>
                </div>
              </div>
            </div>
          </div>
        ) : selected && selectedConv ? (
          <>
            {/* Compact conversation switcher */}
            <div className="grid min-h-0 flex-1 grid-cols-[clamp(220px,25vw,280px)_minmax(0,1fr)]">
            <div className="flex min-w-0 flex-col overflow-hidden border-r border-white/10 bg-slate-900/70">
              <div className="border-b border-white/10 px-4 py-4">
                <div className="text-sm font-semibold text-slate-200">Discussions</div>
                <div className="mt-0.5 text-[11px] text-slate-500">Vos conversations récentes</div>
                <div className="mt-2 text-[10px] font-medium text-slate-600">{filtered.length} discussion{filtered.length === 1 ? '' : 's'}</div>
                <div className="relative mt-3">
                  <span className="absolute left-3 top-2 text-slate-500">⌕</span>
                  <input
                    type="text"
                    placeholder="Rechercher"
                    value={search}
                    onChange={e => setSearch(e.target.value)}
                    className="w-full rounded-xl border border-white/10 bg-slate-950/60 py-2 pl-8 pr-3 text-xs text-slate-100 placeholder-slate-600 focus:border-blue-400/50 focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                  />
                </div>
              </div>
              <div className="max-h-full overflow-y-auto py-2">
                {filtered.map(conv => (
                  <button
                    key={conv.id}
                    onClick={() => selectConversation(conv.id)}
                    className={`flex w-full items-center gap-3 border-l-2 border-b border-white/[0.04] px-3 py-3 text-left transition ${
                      selected === conv.id
                        ? 'border-blue-400 bg-blue-500/15'
                        : 'border-transparent hover:bg-white/[0.05]'
                    }`}
                  >
                    <ConversationAvatar conversation={conv} />
                    <div className="min-w-0">
                      <div className="truncate text-sm font-medium text-slate-100">{formatConversationTitle(conv.title)}</div>
                      <div className="mt-1 truncate text-[11px] text-slate-500">{isTelegram(conv.title) ? 'Telegram' : 'Conversation récente'}</div>
                    </div>
                  </button>
                ))}
              </div>
            </div>
            <div className="flex min-w-0 flex-1 flex-col">
            {/* Chat header */}
            <div className="min-h-[78px] px-4 sm:px-7 py-3 flex items-center gap-3 sm:gap-4 border-b border-white/10 bg-[#111827]">
              <ConversationAvatar conversation={selectedConv} />
              <div className="min-w-0 flex-1">
                <div className="text-[10px] font-medium uppercase tracking-[0.18em] text-blue-400">Socials</div>
                <div className="truncate font-semibold text-slate-100">{formatConversationTitle(selectedConv.title)}</div>
                <div className="text-xs text-slate-500 flex items-center gap-1.5 mt-0.5"><span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />Synchronisé avec {getConnectorIcon(selectedConv.title) === '✈️' ? 'Telegram' : 'Socials'}</div>
              </div>
              <button
                className="hidden h-9 w-9 shrink-0 items-center justify-center rounded-xl border border-white/10 bg-white/5 text-slate-400 transition hover:bg-white/10 hover:text-white sm:flex"
                aria-label="Options de la conversation"
                title="Options"
              >
                •••
              </button>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto space-y-1 bg-[#0b1220] p-4 sm:px-7 sm:py-8">
              {messages.map((msg, i) => {
                const isMe = msg.connector_id === 'Vous'
                const showSender = i === 0 || messages[i - 1]?.connector_id !== msg.connector_id
                return (
                  <div key={msg.id} className={`flex flex-col ${isMe ? 'items-end' : 'items-start'} ${showSender ? 'mt-3' : ''}`}>
                    {showSender && (
                      <div className={`text-xs text-gray-500 mb-1 px-1 ${isMe ? 'text-right w-full' : ''}`}>
                        {msg.connector_id}
                      </div>
                    )}
                    <div
                      className={`max-w-[85%] sm:max-w-md px-4 py-2.5 rounded-2xl text-sm leading-relaxed break-words ${
                        isMe
                          ? 'bg-blue-600 text-white rounded-br-md'
                          : 'bg-slate-800/90 text-slate-100 rounded-bl-md border border-slate-700/60'
                      }`}
                    >
                      {msg.content}
                      <div className={`text-[10px] mt-1 ${isMe ? 'text-blue-200' : 'text-gray-500'}`}>
                        {new Date(msg.created_at).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })}
                      </div>
                    </div>
                  </div>
                )
              })}
              <div ref={messagesEndRef} />
            </div>

            {/* Input */}
            <div className="border-t border-white/10 bg-[#111827] p-3 pb-[4.5rem] sm:px-7 sm:py-5 sm:pb-[4.5rem]">
              <div className="flex gap-2 rounded-2xl border border-white/10 bg-slate-950/50 p-1.5 shadow-inner shadow-black/20">
                <input
                  type="text"
                  value={newMsg}
                  onChange={e => setNewMsg(e.target.value)}
                  onKeyDown={e => e.key === 'Enter' && handleSend()}
                  placeholder="Ecrire un message..."
                  className="min-w-0 flex-1 bg-transparent px-3 sm:px-4 py-3 text-sm placeholder-slate-500 focus:outline-none"
                />
                <button
                  onClick={handleSend}
                  className="shrink-0 rounded-xl bg-blue-600 px-4 sm:px-6 py-3 text-sm font-medium shadow-lg shadow-blue-950/30 transition-all hover:bg-blue-500 hover:-translate-y-0.5"
                >
                  Envoyer
                </button>
              </div>
            </div>
            </div>
            </div>
          </>
        ) : (
          <div className="grid min-h-0 flex-1 grid-cols-[clamp(220px,25vw,280px)_minmax(0,1fr)]">
            <div className="flex min-w-0 flex-col overflow-hidden border-r border-white/10 bg-slate-900/70">
              <div className="border-b border-white/10 px-4 py-3">
                <div className="text-xs font-semibold uppercase tracking-[0.16em] text-slate-400">Conversations</div>
                <div className="mt-1 text-[11px] text-slate-600">{filtered.length} discussion{filtered.length === 1 ? '' : 's'}</div>
                <div className="relative mt-3">
                  <span className="absolute left-3 top-2 text-slate-500">⌕</span>
                  <input
                    type="text"
                    placeholder="Rechercher"
                    value={search}
                    onChange={e => setSearch(e.target.value)}
                    className="w-full rounded-xl border border-white/10 bg-slate-950/60 py-2 pl-8 pr-3 text-xs text-slate-100 placeholder-slate-600 focus:border-blue-400/50 focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                  />
                </div>
              </div>
              <div className="overflow-y-auto py-2">
                {filtered.map(conv => (
                  <button
                    key={conv.id}
                    onClick={() => selectConversation(conv.id)}
                    className="flex w-full items-center gap-3 border-l-2 border-transparent px-3 py-3 text-left transition hover:bg-white/[0.05]"
                  >
                    <ConversationAvatar conversation={conv} />
                    <div className="min-w-0">
                      <div className="truncate text-sm font-medium text-slate-100">{formatConversationTitle(conv.title)}</div>
                      <div className="mt-1 truncate text-[11px] text-slate-500">{isTelegram(conv.title) ? 'Telegram' : 'Conversation récente'}</div>
                    </div>
                  </button>
                ))}
              </div>
            </div>
            <div className="flex min-w-0 flex-1 flex-col overflow-y-auto bg-[#0f172a] p-5 pt-24 sm:p-10 sm:pt-24">
            <div className="mx-auto max-w-3xl">
              <div className="mb-8">
                <div className="text-xs font-medium uppercase tracking-[0.18em] text-blue-400">Socials</div>
                <h2 className="mt-2 text-2xl font-semibold text-slate-100">Votre espace de conversations</h2>
                <p className="mt-2 text-sm text-slate-500">Sélectionnez une discussion dans la colonne de gauche pour commencer.</p>
              </div>
              <div className="rounded-2xl border border-dashed border-slate-700 bg-slate-900/40 p-10 text-center">
                <div className="text-3xl text-slate-600">▤</div>
                <div className="mt-3 font-medium text-slate-300">Aucune discussion ouverte</div>
                <div className="mt-1 text-sm text-slate-500">Choisissez une conversation dans la colonne de gauche.</div>
              </div>
            </div>
            </div>
          </div>
        )}
      </div>

      {activeSection === 'conversations' && !selected && (
        <>
          {showNewChat && (
            <div className="fixed bottom-28 right-5 z-50 w-[min(360px,calc(100vw-2.5rem))] rounded-2xl border border-white/10 bg-slate-900/95 p-4 shadow-[0_20px_55px_rgba(2,6,23,0.6)] backdrop-blur-xl">
              <div className="mb-3 flex items-center justify-between">
                <span className="text-sm font-semibold text-slate-200">Nouvelle conversation</span>
                <button
                  onClick={() => { setShowNewChat(false); setNewChatTitle('') }}
                  className="rounded-lg px-2 py-1 text-slate-500 transition-colors hover:bg-white/10 hover:text-white"
                  aria-label="Fermer"
                >
                  ×
                </button>
              </div>
              <div className="mb-3 rounded-xl border border-blue-900/40 bg-blue-950/30 p-3 text-xs text-slate-400">
                <span className="font-medium text-blue-300">Telegram :</span> une conversation apparaît automatiquement après le premier message reçu.
              </div>
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder="Nom de la conversation..."
                  value={newChatTitle}
                  onChange={e => setNewChatTitle(e.target.value)}
                  onKeyDown={e => e.key === 'Enter' && handleCreateChat()}
                  className="min-w-0 flex-1 rounded-xl border border-white/10 bg-slate-950/70 px-3 py-2.5 text-sm placeholder-slate-500 focus:border-blue-400/50 focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                  autoFocus
                />
                <button
                  onClick={handleCreateChat}
                  className="rounded-xl bg-blue-600 px-3 py-2 text-sm font-medium transition-colors hover:bg-blue-500"
                >
                  Créer
                </button>
              </div>
            </div>
          )}
          <button
            onClick={() => setShowNewChat(value => !value)}
            className="fixed bottom-24 right-6 z-40 flex h-14 w-14 items-center justify-center rounded-2xl bg-blue-600 text-2xl font-light text-white shadow-[0_12px_30px_rgba(37,99,235,0.45)] transition-all hover:-translate-y-1 hover:bg-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-300/70"
            title="Nouvelle discussion"
            aria-label="Nouvelle discussion"
          >
            {showNewChat ? '×' : '+'}
          </button>
        </>
      )}

      {/* Bottom navigation */}
      <nav
        className="fixed bottom-3 left-1/2 z-40 h-[58px] w-[min(400px,calc(100vw-1.5rem))] -translate-x-1/2 rounded-[20px] border border-white/10 bg-slate-900/85 p-1.5 shadow-[0_14px_36px_rgba(2,6,23,0.5),0_3px_10px_rgba(15,23,42,0.3)] backdrop-blur-xl"
        aria-label="Navigation principale"
      >
        <div className="flex h-full items-stretch justify-around gap-0.5">
          <button
            onClick={() => openSection('conversations')}
            className={`relative flex min-w-0 flex-1 flex-col items-center justify-center gap-0.5 rounded-[15px] text-[10px] font-medium transition-all ${
              activeSection === 'conversations' ? 'bg-blue-500/20 text-blue-300 shadow-inner shadow-blue-400/10' : 'text-slate-500 hover:bg-white/5 hover:text-slate-200'
            }`}
          >
            <span className="text-base leading-none">▤</span>
            <span>Conversations</span>
            {activeSection === 'conversations' && <span className="absolute bottom-1 h-1 w-1 rounded-full bg-blue-400 shadow-[0_0_8px_rgba(96,165,250,0.9)]" />}
          </button>
          <button
            onClick={() => openSection('channels')}
            className={`relative flex min-w-0 flex-1 flex-col items-center justify-center gap-0.5 rounded-[15px] text-[10px] font-medium transition-all ${
              activeSection === 'channels' ? 'bg-blue-500/20 text-blue-300 shadow-inner shadow-blue-400/10' : 'text-slate-500 hover:bg-white/5 hover:text-slate-200'
            }`}
          >
            <span className="text-base leading-none">⌁</span>
            <span>Canaux</span>
            {activeSection === 'channels' && <span className="absolute bottom-1 h-1 w-1 rounded-full bg-blue-400 shadow-[0_0_8px_rgba(96,165,250,0.9)]" />}
          </button>
          <button
            onClick={() => openSection('settings')}
            className={`relative flex min-w-0 flex-1 flex-col items-center justify-center gap-0.5 rounded-[15px] text-[10px] font-medium transition-all ${
              activeSection === 'settings' ? 'bg-blue-500/20 text-blue-300 shadow-inner shadow-blue-400/10' : 'text-slate-500 hover:bg-white/5 hover:text-slate-200'
            }`}
          >
            <span className="text-base leading-none">⚙</span>
            <span>Paramètres</span>
            {activeSection === 'settings' && <span className="absolute bottom-1 h-1 w-1 rounded-full bg-blue-400 shadow-[0_0_8px_rgba(96,165,250,0.9)]" />}
          </button>
        </div>
      </nav>
    </div>
  )
}
