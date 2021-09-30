import "./OrderBook.css";

type Order = {
  size: number;
  price: number;
};

function OrderBook({
  sellEntries,
  buyEntries,
}: {
  sellEntries: Order[];
  buyEntries: Order[];
}) {
  sellEntries.sort((a, b) => b.price - a.price);
  sellEntries = sellEntries.slice(
    Math.max(0, sellEntries.length - 10),
    sellEntries.length
  );
  buyEntries.sort((a, b) => b.price - a.price);
  buyEntries = buyEntries.slice(0, 10);
  let spread = 0;
  if (sellEntries.length > 0 && buyEntries.length > 0) {
    spread = sellEntries[sellEntries.length - 1].price - buyEntries[0].price;
  }
  return (
    <div className="root">
      <div className="header">Order Book</div>
      <div className="orderBookContainer">
        <div className="labelContainer">
          <div className="leftLabelCol">
            <span>Market Size</span>
          </div>
          <div className="rightLabelCol">
            <span>Price (USD)</span>
          </div>
        </div>
        <div className="tableContainer">
          <OrderBookTableEntries entries={sellEntries} side={"sell"} />
          <div className="labelContainer">
            <div className="leftLabelCol">
              <span>USD Spread</span>
            </div>
            <div className="rightLabelCol">
              <span>{spread.toFixed(2)}</span>
            </div>
          </div>
          <OrderBookTableEntries entries={buyEntries} side={"buy"} />
        </div>
      </div>
    </div>
  );
}

function OrderBookTableEntries({
  side,
  entries,
}: {
  side: string;
  entries: Order[];
}) {
  return (
    <div className="entriesContainer">
      {entries.map((order, i) => (
        <div key={`${side}-${i}`} className="entryContainer">
          <div className="leftLabelCol">
            <span className="entryText">{order.size}</span>
          </div>
          <div className="rightLabelCol">
            <span
              className={`${
                side === "sell" ? "entrySellText" : "entryBuyText"
              }`}
            >
              {order.price.toFixed(2)}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}

export default OrderBook;
