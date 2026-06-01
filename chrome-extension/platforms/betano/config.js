const BETANO_CONFIG = {
  name: 'betano',
  label: 'Betano',

  matchPatterns: ['https://www.betano.pt/sport/futebol/jogos-de-hoje/*'],

  pollIntervalMs: 30000,

  endpoints: {
    ssrBlocks: '/api/sport/futebol/jogos-de-hoje/?req=s,stnf,c,mb,mbl',
  },

  marketTypeNames: {
    MRES: 'Resultado Final',
    HCTG: 'Total de Golos Mais/Menos',
    BTSC: 'Ambas as Equipas Marcam',
    DBLC: 'Hipótese Dupla',
    OUH1: 'Total de Golos 1ª Parte',
    DNOB: 'Empate Anula Aposta',
  },

  parseBlocks(data) {
    const blocks = data?.blocks ?? [];
    let totalEvents = 0;
    let totalMarkets = 0;

    for (const block of blocks) {
      const events = block.events ?? [];
      totalEvents += events.length;
      for (const event of events) {
        totalMarkets += (event.markets ?? []).length;
      }
    }

    return {
      leagues: blocks.length,
      events: totalEvents,
      markets: totalMarkets,
      blocks,
    };
  },
};
