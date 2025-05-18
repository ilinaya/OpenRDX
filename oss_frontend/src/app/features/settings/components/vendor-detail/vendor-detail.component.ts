import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { VendorService } from '../../../../shared/services/vendor.service';
import { Vendor } from '../../../../shared/models/vendor.model';
import {DatePipe} from '@angular/common';

@Component({
  selector: 'app-vendor-detail',
  templateUrl: './vendor-detail.component.html',
  imports: [
    DatePipe,
  ],
  styleUrls: ['./vendor-detail.component.scss'],
})
export class VendorDetailComponent implements OnInit {
  vendor?: Vendor;
  loading = false;
  error = '';

  constructor(
    private vendorService: VendorService,
    private router: Router,
    private route: ActivatedRoute
  ) {}

  ngOnInit(): void {
    const vendorId = this.route.snapshot.params['id'];
    if (vendorId) {
      this.loadVendor(vendorId);
    }
  }

  loadVendor(id: number): void {
    this.loading = true;
    this.vendorService.getVendor(id).subscribe({
      next: (vendor) => {
        this.vendor = vendor;
        this.loading = false;
      },
      error: (err) => {
        this.error = 'Failed to load vendor. Please try again later.';
        console.error('Error loading vendor:', err);
        this.loading = false;
      }
    });
  }

  editVendor(): void {
    if (this.vendor) {
      this.router.navigate(['edit'], { relativeTo: this.route });
    }
  }

  deleteVendor(): void {
    if (!this.vendor) return;

    if (confirm('Are you sure you want to delete this vendor?')) {
      this.vendorService.deleteVendor(this.vendor.id).subscribe({
        next: () => {
          this.router.navigate(['../'], { relativeTo: this.route });
        },
        error: (err) => {
          this.error = 'Failed to delete vendor. Please try again later.';
          console.error('Error deleting vendor:', err);
        }
      });
    }
  }

  goBack(): void {
    this.router.navigate(['../'], { relativeTo: this.route });
  }
}
