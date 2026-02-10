// ============================================
// KOD Explorer - Main Application
// ============================================

import { ApiPromise, WsProvider } from '@polkadot/api';
import { u8aToHex } from '@polkadot/util';
import nacl from 'tweetnacl';

// ============================================
// STATE
// ============================================

let api = null;
let unsubBlock = null;

// ============================================
// DOM REFERENCES
// ============================================

const $ = (id) => document.getElementById(id);

const els = {
  wsInput: $('wsInput'),
  btnConnect: $('btnConnect'),
  connectionStatus: $('connectionStatus'),
  chainInfo: $('chainInfo'),
  chainName: $('chainName'),
  bestBlock: $('bestBlock'),
  statsSection: $('statsSection'),
  statTotalTrades: $('statTotalTrades'),
  statCompleted: $('statCompleted'),
  statVolume: $('statVolume'),
  statListings: $('statListings'),
  querySection: $('querySection'),
  tradeIdInput: $('tradeIdInput'),
  btnQuery: $('btnQuery'),
  queryError: $('queryError'),
  tradeResult: $('tradeResult'),
  decryptSection: $('decryptSection'),
  seedInput: $('seedInput'),
  btnDecrypt: $('btnDecrypt'),
  decryptError: $('decryptError'),
  decryptResult: $('decryptResult'),
};

// ============================================
// HELPERS
// ============================================

function show(el) { el.classList.remove('hidden'); }
function hide(el) { el.classList.add('hidden'); }

function setStatus(state, text) {
  const dot = els.connectionStatus.querySelector('.status-dot');
  const txt = els.connectionStatus.querySelector('.status-text');
  dot.className = 'status-dot ' + state;
  txt.textContent = text;
}

function shortAddr(addr) {
  if (!addr) return '-';
  const s = addr.toString();
  return s.length > 16 ? s.slice(0, 8) + '...' + s.slice(-6) : s;
}

function hexToBytes(hex) {
  const clean = hex.startsWith('0x') ? hex.slice(2) : hex;
  const bytes = new Uint8Array(clean.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(clean.substr(i * 2, 2), 16);
  }
  return bytes;
}

function formatBalance(raw) {
  // KOD has 18 decimals (UNIT = 10^18)
  const str = raw.toString();
  if (str === '0') return '0';
  const padded = str.padStart(19, '0');
  const whole = padded.slice(0, -18) || '0';
  const frac = padded.slice(-18).replace(/0+$/, '');
  return frac ? `${whole}.${frac.slice(0, 4)}` : whole;
}

function formatBalanceShort(raw) {
  const str = raw.toString();
  if (str === '0') return '0';
  const padded = str.padStart(19, '0');
  const whole = padded.slice(0, -18) || '0';
  const frac = padded.slice(-18, -16);
  const num = parseFloat(`${whole}.${frac}`);
  if (num >= 1_000_000) return (num / 1_000_000).toFixed(1) + 'M';
  if (num >= 1_000) return (num / 1_000).toFixed(1) + 'K';
  return num.toString();
}

function showToast(msg) {
  let toast = document.querySelector('.copy-toast');
  if (!toast) {
    toast = document.createElement('div');
    toast.className = 'copy-toast';
    document.body.appendChild(toast);
  }
  toast.textContent = msg;
  toast.classList.add('show');
  setTimeout(() => toast.classList.remove('show'), 1500);
}

function copyToClipboard(text) {
  navigator.clipboard.writeText(text).then(() => {
    showToast('Kopyalandi!');
  }).catch(() => {});
}

// TradeStatus enum mapping (matches Rust enum order)
const TRADE_STATUS = {
  PendingSellerConfirm: { label: 'Satici Onayi Bekleniyor', class: 'badge-pending' },
  Escrow: { label: 'Escrow', class: 'badge-escrow' },
  Completed: { label: 'Tamamlandi', class: 'badge-completed' },
  Disputed: { label: 'Anlaşmazlik', class: 'badge-disputed' },
  Refunded: { label: 'Iade Edildi', class: 'badge-refunded' },
};

function getStatusInfo(status) {
  // status is a codec enum, .type gives the variant name
  const key = status.type || status.toString();
  return TRADE_STATUS[key] || { label: key, class: 'badge-pending' };
}

// ============================================
// CONNECTION
// ============================================

let currentTradeId = null;
let currentEncryptedData = null;
let currentUserKey = null;

async function connect() {
  const wsUrl = els.wsInput.value.trim();
  if (!wsUrl) return;

  // Disconnect existing
  if (api) {
    try {
      if (unsubBlock) unsubBlock();
      await api.disconnect();
    } catch (_) {}
    api = null;
  }

  setStatus('connecting', 'Baglaniliyor...');
  els.btnConnect.disabled = true;
  els.btnConnect.textContent = 'Baglaniyor...';

  try {
    const provider = new WsProvider(wsUrl);
    api = await ApiPromise.create({ provider });

    // Chain info
    const chain = await api.rpc.system.chain();
    els.chainName.textContent = chain.toString();

    // Subscribe to new heads
    unsubBlock = await api.rpc.chain.subscribeNewHeads((header) => {
      els.bestBlock.textContent = header.number.toNumber().toLocaleString();
    });

    setStatus('online', 'Bagli');
    show(els.chainInfo);
    els.btnConnect.textContent = 'Bagli';

    // Load stats and show sections
    await loadStats();
    show(els.statsSection);
    show(els.querySection);

    // Handle disconnect
    provider.on('disconnected', () => {
      setStatus('offline', 'Baglanti kesildi');
      els.btnConnect.disabled = false;
      els.btnConnect.textContent = 'Baglan';
      hide(els.statsSection);
      hide(els.querySection);
      hide(els.decryptSection);
    });

  } catch (err) {
    console.error('Connection error:', err);
    setStatus('offline', 'Baglanti basarisiz: ' + err.message);
    els.btnConnect.disabled = false;
    els.btnConnect.textContent = 'Baglan';
    return;
  }

  els.btnConnect.disabled = false;
}

// ============================================
// STATS
// ============================================

async function loadStats() {
  if (!api) return;

  try {
    const [nextTradeId, completed, volume, nextListingId] = await Promise.all([
      api.query.trade.nextTradeId(),
      api.query.trade.totalTradesCompleted(),
      api.query.trade.totalVolume(),
      api.query.trade.nextListingId(),
    ]);

    els.statTotalTrades.textContent = nextTradeId.toNumber().toLocaleString();
    els.statCompleted.textContent = completed.toNumber().toLocaleString();
    els.statVolume.textContent = formatBalanceShort(volume);
    els.statListings.textContent = nextListingId.toNumber().toLocaleString();
  } catch (err) {
    console.error('Stats error:', err);
  }
}

// ============================================
// TRADE QUERY
// ============================================

async function queryTrade() {
  const tradeIdStr = els.tradeIdInput.value.trim();
  if (!tradeIdStr) return;
  if (!api) return;

  const tradeId = parseInt(tradeIdStr, 10);
  hide(els.queryError);
  hide(els.tradeResult);
  hide(els.decryptSection);

  els.btnQuery.disabled = true;
  els.btnQuery.textContent = 'Sorgulanıyor...';

  try {
    // Query trade
    const tradeOpt = await api.query.trade.trades(tradeId);

    if (tradeOpt.isNone) {
      showError(els.queryError, `Trade #${tradeId} bulunamadi.`);
      els.btnQuery.disabled = false;
      els.btnQuery.textContent = 'Sorgula';
      return;
    }

    const trade = tradeOpt.unwrap();
    currentTradeId = tradeId;

    // Check for encrypted contract
    const encOpt = await api.query.trade.encryptedContracts(tradeId);
    const hasEncrypted = encOpt.isSome;
    currentEncryptedData = hasEncrypted ? encOpt.unwrap() : null;

    // Check for diagnostics
    let diagReport = null;
    const hasDiag = (await api.query.trade.hasDiagnostics(tradeId)).isTrue;
    if (hasDiag) {
      const diagOpt = await api.query.trade.diagnosticReports(tradeId);
      if (diagOpt.isSome) {
        diagReport = diagOpt.unwrap();
      }
    }

    // Build result HTML
    renderTradeResult(trade, tradeId, hasEncrypted, diagReport);
    show(els.tradeResult);

    // Show decrypt section if encrypted
    if (hasEncrypted) {
      show(els.decryptSection);
      hide(els.decryptResult);
      hide(els.decryptError);
      els.seedInput.value = '';
    }

  } catch (err) {
    console.error('Query error:', err);
    showError(els.queryError, 'Sorgulama hatasi: ' + err.message);
  }

  els.btnQuery.disabled = false;
  els.btnQuery.textContent = 'Sorgula';
}

function showError(el, msg) {
  el.textContent = msg;
  show(el);
}

function renderTradeResult(trade, tradeId, hasEncrypted, diagReport) {
  const statusInfo = getStatusInfo(trade.status);

  let html = `
    <div class="trade-header">
      <h3>Trade #${tradeId}</h3>
      <span class="badge ${statusInfo.class}">${statusInfo.label}</span>
      ${hasEncrypted ? '<span class="badge badge-encrypted">Sifreli Sozlesme Mevcut</span>' : ''}
    </div>
    <div class="trade-details">
      <div class="detail-item">
        <div class="detail-label">Alici</div>
        <div class="detail-value" data-copy="${trade.buyer.toString()}" title="Kopyalamak icin tiklayin">${shortAddr(trade.buyer)}</div>
      </div>
      <div class="detail-item">
        <div class="detail-label">Satici</div>
        <div class="detail-value" data-copy="${trade.seller.toString()}" title="Kopyalamak icin tiklayin">${shortAddr(trade.seller)}</div>
      </div>
      <div class="detail-item">
        <div class="detail-label">Fiyat</div>
        <div class="detail-value no-copy">${formatBalance(trade.price)} KOD</div>
      </div>
      <div class="detail-item">
        <div class="detail-label">Ilan ID</div>
        <div class="detail-value no-copy">#${trade.listingId.toString()}</div>
      </div>
      <div class="detail-item">
        <div class="detail-label">Alici Teminati</div>
        <div class="detail-value no-copy">${formatBalance(trade.buyerBond)} KOD</div>
      </div>
      <div class="detail-item">
        <div class="detail-label">Satici Teminati</div>
        <div class="detail-value no-copy">${formatBalance(trade.sellerBond)} KOD</div>
      </div>
      <div class="detail-item full-width">
        <div class="detail-label">Sozlesme Hash</div>
        <div class="detail-value" data-copy="${u8aToHex(trade.contractHash)}" title="Kopyalamak icin tiklayin">${u8aToHex(trade.contractHash)}</div>
      </div>
  `;

  // Delivery attestation hash
  if (trade.deliveryAttestationHash && trade.deliveryAttestationHash.isSome) {
    const h = u8aToHex(trade.deliveryAttestationHash.unwrap());
    html += `
      <div class="detail-item full-width">
        <div class="detail-label">Teslimat Attestation Hash</div>
        <div class="detail-value" data-copy="${h}" title="Kopyalamak icin tiklayin">${h}</div>
      </div>
    `;
  }

  // Final hash
  if (trade.finalHash && trade.finalHash.isSome) {
    const h = u8aToHex(trade.finalHash.unwrap());
    html += `
      <div class="detail-item full-width">
        <div class="detail-label">Final Hash</div>
        <div class="detail-value" data-copy="${h}" title="Kopyalamak icin tiklayin">${h}</div>
      </div>
    `;
  }

  // Created at
  html += `
      <div class="detail-item">
        <div class="detail-label">Olusturulma Blogu</div>
        <div class="detail-value no-copy">#${trade.createdAt.toString()}</div>
      </div>
  `;

  // Diagnostics
  if (diagReport) {
    const score = diagReport.score.toNumber();
    const passed = diagReport.passedCount.toNumber();
    const failed = diagReport.failedCount.toNumber();
    const total = passed + failed;
    html += `
      <div class="detail-item full-width">
        <div class="detail-label">Tanilama Raporu</div>
        <div class="diag-bar-container">
          <div class="diag-bar">
            <div class="diag-bar-fill" style="width: ${score}%"></div>
          </div>
          <div class="diag-info">
            <span>Skor: ${score}/100</span>
            <span>Gecen: ${passed} / Toplam: ${total}</span>
          </div>
        </div>
      </div>
    `;
  }

  html += '</div>';

  els.tradeResult.innerHTML = html;

  // Click to copy
  els.tradeResult.querySelectorAll('.detail-value[data-copy]').forEach(el => {
    el.addEventListener('click', () => copyToClipboard(el.dataset.copy));
  });
}

// ============================================
// DECRYPTION
// ============================================

async function decryptContract() {
  hide(els.decryptError);
  hide(els.decryptResult);

  const seedPhrase = els.seedInput.value.trim();
  if (!seedPhrase) {
    showError(els.decryptError, '12 kurtarma kelimenizi girin.');
    return;
  }

  const words = seedPhrase.split(/\s+/);
  if (words.length !== 12) {
    showError(els.decryptError, `12 kelime bekleniyor, ${words.length} kelime girdiniz.`);
    return;
  }

  if (!currentEncryptedData || currentTradeId === null) {
    showError(els.decryptError, 'Sifreli sozlesme verisi bulunamadi.');
    return;
  }

  els.btnDecrypt.disabled = true;
  els.btnDecrypt.textContent = 'Desifre ediliyor...';

  try {
    // Dynamically import key derivation utils
    const { mnemonicToMiniSecret, cryptoWaitReady } = await import('@polkadot/util-crypto');
    await cryptoWaitReady();

    // Derive x25519 keypair from seed phrase
    const miniSecret = mnemonicToMiniSecret(seedPhrase);
    const x25519Keypair = nacl.box.keyPair.fromSecretKey(miniSecret.slice(0, 32));

    // Get the user's wrapped key from chain
    // We need to determine the user's sr25519 address to query the key
    const { Keyring } = await import('@polkadot/keyring');
    const keyring = new Keyring({ type: 'sr25519' });
    const pair = keyring.addFromMnemonic(seedPhrase);
    const userAddress = pair.address;

    // Query the wrapped key for this user
    const wrappedKeyOpt = await api.query.trade.contractEncryptionKeys(currentTradeId, userAddress);

    if (wrappedKeyOpt.isNone) {
      showError(els.decryptError, 'Bu trade icin sifreleme anahtariniz bulunamadi. Bu trade\'in taraflarindan biri olmaniz gerekir.');
      els.btnDecrypt.disabled = false;
      els.btnDecrypt.textContent = 'Desifre Et';
      return;
    }

    const wrappedKeyHex = wrappedKeyOpt.unwrap();
    const wrappedKeyBytes = hexToBytes(wrappedKeyHex.toHex ? wrappedKeyHex.toHex() : wrappedKeyHex.toString());

    // Unwrap: ephemeralPubKey(32) + nonce(24) + encryptedKey(48)
    if (wrappedKeyBytes.length < 32 + 24 + 48) {
      showError(els.decryptError, 'Sarili anahtar verisi gecersiz format.');
      els.btnDecrypt.disabled = false;
      els.btnDecrypt.textContent = 'Desifre Et';
      return;
    }

    const ephemeralPub = wrappedKeyBytes.slice(0, 32);
    const keyNonce = wrappedKeyBytes.slice(32, 56);
    const encryptedSymKey = wrappedKeyBytes.slice(56);

    // NaCl box open to get symmetric key
    const symmetricKey = nacl.box.open(encryptedSymKey, keyNonce, ephemeralPub, x25519Keypair.secretKey);

    if (!symmetricKey) {
      showError(els.decryptError, 'Anahtar desifreleme basarisiz. Yanlis kurtarma kelimeleri veya bu trade\'in tarafi degilsiniz.');
      els.btnDecrypt.disabled = false;
      els.btnDecrypt.textContent = 'Desifre Et';
      return;
    }

    // Now decrypt the contract content
    const encPayloadHex = currentEncryptedData.toHex ? currentEncryptedData.toHex() : currentEncryptedData.toString();
    const encPayload = hexToBytes(encPayloadHex);

    // Payload: nonce(24) + ciphertext
    if (encPayload.length < 24) {
      showError(els.decryptError, 'Sifreli sozlesme verisi gecersiz format.');
      els.btnDecrypt.disabled = false;
      els.btnDecrypt.textContent = 'Desifre Et';
      return;
    }

    const contentNonce = encPayload.slice(0, 24);
    const ciphertext = encPayload.slice(24);

    // NaCl secretbox open
    const plaintext = nacl.secretbox.open(ciphertext, contentNonce, symmetricKey);

    if (!plaintext) {
      showError(els.decryptError, 'Sozlesme desifreleme basarisiz. Veri bozulmus olabilir.');
      els.btnDecrypt.disabled = false;
      els.btnDecrypt.textContent = 'Desifre Et';
      return;
    }

    // Decode JSON
    const decoder = new TextDecoder();
    const contractJson = decoder.decode(plaintext);
    const contract = JSON.parse(contractJson);

    renderDecryptedContract(contract);
    show(els.decryptResult);

  } catch (err) {
    console.error('Decrypt error:', err);
    showError(els.decryptError, 'Desifreleme hatasi: ' + err.message);
  }

  els.btnDecrypt.disabled = false;
  els.btnDecrypt.textContent = 'Desifre Et';
}

function renderDecryptedContract(contract) {
  let html = '';

  // Trade / General info
  if (contract.trade || contract.tradeId) {
    html += `<div class="contract-section">
      <h4>Ticaret Bilgileri</h4>`;
    if (contract.tradeId) html += row('Trade ID', `#${contract.tradeId}`);
    if (contract.trade) {
      const t = contract.trade;
      if (t.tradeId) html += row('Trade ID', `#${t.tradeId}`);
      if (t.listingId) html += row('Ilan ID', `#${t.listingId}`);
      if (t.buyer) html += row('Alici', t.buyer);
      if (t.seller) html += row('Satici', t.seller);
      if (t.price) html += row('Fiyat', t.price);
      if (t.buyerBond) html += row('Alici Teminati', t.buyerBond);
      if (t.sellerBond) html += row('Satici Teminati', t.sellerBond);
      if (t.status) html += row('Durum', t.status);
    }
    html += '</div>';
  }

  // Listing info
  if (contract.listing) {
    html += `<div class="contract-section">
      <h4>Ilan Detaylari</h4>`;
    const l = contract.listing;
    if (l.title) html += row('Baslik', l.title);
    if (l.description) html += row('Aciklama', l.description);
    if (l.category) html += row('Kategori', l.category);
    if (l.brand) html += row('Marka', l.brand);
    if (l.model) html += row('Model', l.model);
    if (l.condition) html += row('Durum', l.condition);
    if (l.price) html += row('Fiyat', l.price);
    html += '</div>';
  }

  // Diagnostics
  if (contract.diagnostics) {
    html += `<div class="contract-section">
      <h4>Tanilama Raporu</h4>`;
    const d = contract.diagnostics;
    if (d.score !== undefined) html += row('Skor', `${d.score}/100`);
    if (d.passedCount !== undefined) html += row('Gecen', d.passedCount);
    if (d.failedCount !== undefined) html += row('Basarisiz', d.failedCount);
    if (d.tests && Array.isArray(d.tests)) {
      d.tests.forEach((test, i) => {
        const result = test.result || test.status || '-';
        const name = test.name || test.testId || `Test ${i + 1}`;
        html += row(name, result);
      });
    }
    html += '</div>';
  }

  // Meeting details
  if (contract.meeting) {
    html += `<div class="contract-section">
      <h4>Bulusma Detaylari</h4>`;
    const m = contract.meeting;
    if (m.location) html += row('Konum', m.location);
    if (m.date) html += row('Tarih', m.date);
    if (m.time) html += row('Saat', m.time);
    if (m.notes) html += row('Notlar', m.notes);
    html += '</div>';
  }

  // Device info
  if (contract.device) {
    html += `<div class="contract-section">
      <h4>Cihaz Bilgisi</h4>`;
    const dev = contract.device;
    Object.entries(dev).forEach(([k, v]) => {
      if (v) html += row(k, v.toString());
    });
    html += '</div>';
  }

  // Notes / conditions
  if (contract.notes) {
    html += `<div class="contract-section">
      <h4>Notlar / Kosullar</h4>`;
    if (typeof contract.notes === 'string') {
      html += `<div class="contract-row"><span class="contract-val">${escapeHtml(contract.notes)}</span></div>`;
    } else if (Array.isArray(contract.notes)) {
      contract.notes.forEach((n, i) => {
        html += row(`#${i + 1}`, n);
      });
    }
    html += '</div>';
  }

  // Timestamps
  if (contract.timestamp || contract.createdAt || contract.acceptedAt) {
    html += `<div class="contract-section">
      <h4>Tarihler</h4>`;
    if (contract.timestamp) html += row('Zaman Damgasi', new Date(contract.timestamp).toLocaleString('tr-TR'));
    if (contract.createdAt) html += row('Olusturulma', contract.createdAt);
    if (contract.acceptedAt) html += row('Kabul Edilme', contract.acceptedAt);
    html += '</div>';
  }

  // Fallback: show raw JSON for any unknown fields
  const knownKeys = new Set(['trade', 'tradeId', 'listing', 'diagnostics', 'meeting', 'device', 'notes', 'timestamp', 'createdAt', 'acceptedAt']);
  const extraKeys = Object.keys(contract).filter(k => !knownKeys.has(k));
  if (extraKeys.length > 0) {
    html += `<div class="contract-section">
      <h4>Diger Bilgiler</h4>`;
    extraKeys.forEach(k => {
      const v = contract[k];
      const display = typeof v === 'object' ? JSON.stringify(v, null, 2) : v.toString();
      html += row(k, display);
    });
    html += '</div>';
  }

  els.decryptResult.innerHTML = html;
}

function row(key, value) {
  return `<div class="contract-row">
    <span class="contract-key">${escapeHtml(key)}</span>
    <span class="contract-val">${escapeHtml(String(value))}</span>
  </div>`;
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// ============================================
// EVENT LISTENERS
// ============================================

els.btnConnect.addEventListener('click', connect);
els.btnQuery.addEventListener('click', queryTrade);
els.btnDecrypt.addEventListener('click', decryptContract);

// Enter key support
els.wsInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') connect();
});

els.tradeIdInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') queryTrade();
});
