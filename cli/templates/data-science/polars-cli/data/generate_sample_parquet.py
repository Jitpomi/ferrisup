import pandas as pd
import numpy as np

# Create a sample DataFrame
data = {
    'id': range(1, 101),
    'name': [f'Item {i}' for i in range(1, 101)],
    'value': np.random.rand(100) * 1000,
    'category': np.random.choice(['A', 'B', 'C', 'D'], 100),
    'date': pd.date_range(start='2023-01-01', periods=100),
    'is_active': np.random.choice([True, False], 100)
}

df = pd.DataFrame(data)

# Save as Parquet file
df.to_parquet('example_data_parquet.parquet', index=False)

print("Sample Parquet file created successfully!")
