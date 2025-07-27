/* global use, db */
// MongoDB Playground
// To disable this template go to Settings | MongoDB | Use Default Template For Playground.
// Make sure you are connected to enable completions and to be able to run a playground.
// Use Ctrl+Space inside a snippet or a string literal to trigger completions.
// The result of the last command run in a playground is shown on the results panel.
// By default the first 20 documents will be returned with a cursor.
// Use 'console.log()' to print to the debug output.
// For more documentation on playgrounds please refer to
// https://www.mongodb.com/docs/mongodb-vscode/playgrounds/

// Select the database to use.
use('db_prices');

// Insert a few documents into the sales collection.
// db.getCollection('prices').find({ date: { $gte: new Date('2021-08-20'), $lt: new Date('2021-08-25') } });
// db.getCollection('prices').find({  max_quantity: { $gte: 11 } });

// Find prices with min_quantity > 10 and max_quantity < 200
db.getCollection('prices').find({ 
 sku: { $eq: '0096234303' },
 min_quantity: { $lte: 21 },
 max_quantity: { $gte: 21 },
 start_date: { $lte: ISODate('2025-07-26') },
 end_date: { $gte: ISODate('2025-07-26') },
 offer_prices: { $elemMatch: { currency: 'USD' } } // Correct syntax for querying array elements
}).sort({ "offer_prices.price": 1 }).limit(1); // Sort by price ascending and return only one document

// db.db_prices.aggregate([
//   {
//     $match: {
//       "offer_prices.currency": "USD" // Match documents where any offer_price has currency USD
//     }
//   },
//   {
//     $unwind: "$offer_prices" // Deconstruct the offer_prices array into separate documents
//   },
//   {
//     $match: {
//       "offer_prices.currency": "USD" // Filter again to ensure we only process USD offers
//     }
//   },
//   {
//     $sort: {
//       "offer_prices.price": 1 // Sort by the price of the USD offer in ascending order
//     }
//   },
//   {
//     $group: {
//       _id: "$_id", // Group back by the original document ID
//       sku: { $first: "$0096234303" },
//       start_date: { $first: { $lte: ISODate('2025-07-26') } },
//       end_date: { $first: { $gte: ISODate('2025-07-26') } },
//       min_quantity: { $first: { $lte: 21 } },
//       max_quantity: { $first: { $gte: 21 }   },
//       offer_prices: { $push: "$offer_prices" } // Reconstruct the offer_prices array
//     }
//   }
// ])
// Run a find command to view items sold on April 4th, 2014.
// const salesOnApril4th = db.getCollection('sales').find({
//   date: { $gte: new Date('2014-04-04'), $lt: new Date('2014-04-05') }
// }).count();

// // Print a message to the output window.
// console.log(`${salesOnApril4th} sales occurred in 2014.`);

// // Here we run an aggregation and open a cursor to the results.
// // Use '.toArray()' to exhaust the cursor to return the whole result set.
// // You can use '.hasNext()/.next()' to iterate through the cursor page by page.
// db.getCollection('sales').aggregate([
//   // Find all of the sales that occurred in 2014.
//   { $match: { date: { $gte: new Date('2014-01-01'), $lt: new Date('2015-01-01') } } },
//   // Group the total sales for each product.
//   { $group: { _id: '$item', totalSaleAmount: { $sum: { $multiply: [ '$price', '$quantity' ] } } } }
// ]);
