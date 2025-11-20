import { Component, OnInit } from '@angular/core';
import {Router, RouterLink} from '@angular/router';
import { VendorService } from '../../../../shared/services/vendor.service';
import { Vendor } from '../../../../shared/models/vendor.model';
import { PagedResponse } from '../../../../shared/models/pagination.model';

@Component({
  selector: 'app-vendor-list',
  templateUrl: './vendor-list.component.html',
  imports: [
    RouterLink,
  ],
  styleUrls: ['./vendor-list.component.scss'],
})
export class VendorListComponent implements OnInit {
  vendors: Vendor[] = [];
  loading = false;
  error = '';
  currentPage = 1;
  pageSize = 10;
  totalItems = 0;
  totalPages = 0;

  constructor(
    private vendorService: VendorService,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadVendors();
  }

  loadVendors(): void {
    this.loading = true;
    this.vendorService.getAllVendors({
      page: this.currentPage,
      page_size: this.pageSize
    }).subscribe({
      next: (response: PagedResponse<Vendor>) => {
        this.vendors = response.results;
        this.totalItems = response.count;
        this.totalPages = Math.ceil(response.count / this.pageSize);
        this.loading = false;
      },
      error: (err) => {
        this.error = 'Failed to load vendors. Please try again later.';
        console.error('Error loading vendors:', err);
        this.loading = false;
      }
    });
  }

  onPageChange(page: number): void {
    this.currentPage = page;
    this.loadVendors();
  }

  createVendor(): void {
    this.router.navigate(['vendors', 'new']);
  }

  viewVendor(id: number): void {
    this.router.navigate(['vendors', id]);
  }

  editVendor(id: number): void {
    this.router.navigate(['vendors', id, 'edit']);
  }

  deleteVendor(id: number): void {
    if (confirm('Are you sure you want to delete this vendor?')) {
      this.vendorService.deleteVendor(id).subscribe({
        next: () => {
          this.loadVendors();
        },
        error: (err) => {
          this.error = 'Failed to delete vendor. Please try again later.';
          console.error('Error deleting vendor:', err);
        }
      });
    }
  }
}
